#![doc(test(attr(deny(warnings))))]
#![warn(missing_docs)]
// Forbid unsafe code in the actual code, but tests use libc::kill.
#![cfg_attr(not(test), forbid(unsafe_code))]

//!  A tiny `Read`/`Write` wrapper that can reopen the underlying IO object.
//!
//! The main motivation is integration of logging with logrotate. Usually, when
//! logrotate wants to rotate log files, it moves the current log file to a new
//! place and creates a new empty file. However, for the new messages to appear in
//! the new file, a running program needs to close and reopen the file. This is
//! most often signalled by `SIGHUP`.
//!
//! # Traits
//!
//! The amount of supported traits is somewhat limited. For example, [BufRead][std::io::BufRead] or
//! [Seek][std::io::Seek] are not implemented, because the behavior across reopens would be
//! confusing if not outright wrong.
//!
//! # Features
//!
//! The `signals` feature adds support to registering a reopening as a result of received a signal
//! (for example the `SIGHUP` one).
//!
//! # Examples
//!
//! This allows reopening the IO object used inside the logging drain at runtime.
//!
//! ```rust,no_run
//! use std::fs::{File, OpenOptions};
//! use std::io::Error;
//!
//! use log::info;
//! use reopen::Reopen;
//!
//! fn open() -> Result<File, Error> {
//!     OpenOptions::new()
//!         .create(true)
//!         .write(true)
//!         .append(true)
//!         .open("/log/file")
//! }
//!
//! fn main() -> Result<(), Error> {
//!     let file = Reopen::new(Box::new(&open))?;
//! # #[cfg(all(feature = "signals", not(windows)))]
//!     file.handle().register_signal(signal_hook::consts::SIGHUP)?;
//!     simple_logging::log_to(file, log::LevelFilter::Debug);
//!     info!("Hey, it's logging");
//!     Ok(())
//! }
//! ```
//!
//! Note that this solution is a bit hacky and probably solves only the most common use case.
//!
//! If you find another use case for it, I'd like to hear about it.

use std::fmt::{self, Debug, Formatter, Result as FmtResult};
use std::io::{Error, Read, Write};
#[cfg(vectored)]
use std::io::{IoSlice, IoSliceMut};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[cfg(feature = "signals")]
mod signals;

/// A handle to signal a companion [`Reopen`] object to do a reopen on its next operation.
///
/// Cloning creates interchangeable handles (they all control the same [`Reopen`]). Cloning is
/// cheap (it's only an [`Arc`] in disguise).
#[derive(Clone, Debug)]
pub struct Handle(Arc<AtomicBool>);

impl Handle {
    /// Signals the companion [`Reopen`](struct.Reopen.html) object to do a reopen on its next
    /// operation.
    pub fn reopen(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    /// Creates an unpaired handle, not connected to any ['Reopen'].
    ///
    /// It can be added to a new [`Reopen`] later on with [`with_handle`][Reopen::with_handle].
    pub fn stub() -> Self {
        Handle(Arc::new(AtomicBool::new(false)))
    }
}

/// A `Read`/`Write` proxy that can reopen the underlying object.
///
/// It is constructed with a function that can open a new instance of the object. If it is signaled
/// to reopen it (though [`handle`](#method.handle)), it drops the old instance and uses the
/// function to create a new one at the next IO operation.
///
/// # Error handling
///
/// The reopening is performed lazily, on the first operation done to the object. Opening a new
/// instance can fail with an error. If this happens, the error is returned as part of the
/// operation being performed ‒ therefore, you can get an error like `File not found` while
/// performing `read`.
///
/// If an error happens, the operation is aborted. Next time an operation is performed, another
/// attempt to open the object is made (which in turn can fail again).
///
/// # Scheduling of a reopen
///
/// The implementation tries to ensure whole operations happen on the same FD. For example, even if
/// multiple [`read`][Read::read] calls need to be performed as part of
/// [`read_exact`][Read::read_exact], the [`Reopen`] will check for reopening flags only once
/// before the whole operation and then will keep the same FD.
///
/// If this is not enough, the [`Reopen`] can be [locked][Reopen::lock] to bundle multiple
/// operations without reopening.
///
/// # Handling of ends
///
/// Certain operations make ordinary file descriptors „finished“ ‒ for example,
/// [`read_to_end`][Read::read_to_end]. Usually, further calls to any read operations would produce
/// EOF from then on.
///
/// While this reaches the end of the currently opened FD and further read operations would still
/// produce EOF, reopening the FD may lead to it being readable again. Therefore, reaching EOF is
/// not necessarily final for [`Reopen`].
pub struct Reopen<FD> {
    signal: Arc<AtomicBool>,
    constructor: Box<dyn Fn() -> Result<FD, Error> + Send>,
    fd: Option<FD>,
}

impl<FD> Reopen<FD> {
    /// Creates a new instance.
    pub fn new(constructor: Box<dyn Fn() -> Result<FD, Error> + Send>) -> Result<Self, Error> {
        Self::with_handle(Handle::stub(), constructor)
    }

    /// Creates a new instance from the given handle.
    ///
    /// This might come useful if you want to create the handle beforehand with
    /// [`Handle::stub`](struct.Handle.html#method.stub) (eg. in
    /// [`once_cell`](https://docs.rs/once_cell)).
    ///
    /// Note that using the same handle for multiple `Reopen`s will not work as expected (the first
    /// one to be used resets the signal and the others don't reopen).
    ///
    /// # Examples
    ///
    /// ```
    /// # use reopen::*;
    /// // Something that implements `Write`, for example.
    /// struct Writer;
    ///
    /// let handle = Handle::stub();
    /// let reopen = Reopen::with_handle(handle.clone(), Box::new(|| Ok(Writer)));
    ///
    /// handle.reopen();
    /// # let _ = reopen;
    /// ```
    pub fn with_handle(
        handle: Handle,
        constructor: Box<dyn Fn() -> Result<FD, Error> + Send>,
    ) -> Result<Self, Error> {
        let fd = constructor()?;
        Ok(Self {
            signal: handle.0,
            constructor,
            fd: Some(fd),
        })
    }

    /// Returns a handle to signal this `Reopen` to perform the reopening.
    pub fn handle(&self) -> Handle {
        Handle(Arc::clone(&self.signal))
    }

    /// Lock the [`Reopen`] against reopening in the middle of operation.
    ///
    /// In case of needing to perform multiple operations without reopening in the middle, it can
    /// be locked by this method. This provides access to the inner FD.
    ///
    /// # Errors
    ///
    /// This can result in an error in case the FD needs to be reopened (or wasn't opened
    /// previously) and the reopening results in an error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use std::io::{Error, Write};
    /// # use reopen::Reopen;
    /// # fn main() -> Result<(), Error> {
    /// let mut writer = Reopen::new(Box::new(|| {
    ///     // Vec::<u8> is an in-memory writer
    ///     Ok(Vec::new())
    /// }))?;
    /// let handle = writer.handle();
    /// let mut lock = writer.lock()?;
    /// write!(&mut lock, "Hello ")?;
    ///
    /// // Request reopening. But as we locked, it won't happen until we are done with it.
    /// handle.reopen();
    ///
    /// write!(&mut lock, "world")?;
    ///
    /// // See? Both writes are here now.
    /// assert_eq!(b"Hello world", &lock[..]);
    ///
    /// // But when we return to using the writer directly (and drop the lock by that), it gets
    /// // reopened and we get a whole new Vec to play with.
    /// write!(&mut writer, "Another message")?;
    /// assert_eq!(b"Another message", &writer.lock()?[..]);
    /// # Ok(()) }
    /// ```
    pub fn lock(&mut self) -> Result<&mut FD, Error> {
        if self.signal.swap(false, Ordering::Relaxed) {
            self.fd.take();
        }
        if self.fd.is_none() {
            self.fd = Some((self.constructor)()?);
        }
        Ok(self.fd.as_mut().unwrap())
    }
}

impl<FD: Debug> Debug for Reopen<FD> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("Reopen")
            .field("signal", &self.signal)
            .field("fd", &self.fd)
            .field("constructor", &"...")
            .finish()
    }
}

impl<FD: Read> Read for Reopen<FD> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        let fd = self.lock()?;
        fd.read(buf)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        let fd = self.lock()?;
        fd.read_exact(buf)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize, Error> {
        let fd = self.lock()?;
        fd.read_to_end(buf)
    }

    fn read_to_string(&mut self, buf: &mut String) -> Result<usize, Error> {
        let fd = self.lock()?;
        fd.read_to_string(buf)
    }

    #[cfg(vectored)]
    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> Result<usize, Error> {
        let fd = self.check()?;
        fd.read_vectored(bufs)
    }
}

impl<FD: Write> Write for Reopen<FD> {
    fn flush(&mut self) -> Result<(), Error> {
        let fd = self.lock()?;
        fd.flush()
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        let fd = self.lock()?;
        fd.write(buf)
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<(), Error> {
        let fd = self.lock()?;
        fd.write_all(buf)
    }

    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> Result<(), Error> {
        let fd = self.lock()?;
        fd.write_fmt(fmt)
    }

    #[cfg(vectored)]
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> Result<usize, Error> {
        let fd = self.check()?;
        fd.write_vectored(bufs)
    }
}
