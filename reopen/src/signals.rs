use std::io::Error;
use std::sync::Arc;

use signal_hook::SigId;

use super::Handle;

impl Handle {
    /// Installs a signal handler to invoke the reopening when a certain signal comes.
    ///
    /// # Features
    ///
    /// This is available only with the `signals` feature enabled.
    ///
    /// # Notes
    ///
    /// * Under the hood, this uses the [`signal-hook`](https://crates.io/signal-hook) crate, so
    ///   the same signal can be shared with other actions (to eg. also reload a configuration).
    /// * The same restrictions, errors and panics as in the case of
    ///   [`signal_hook::register`](https://docs.rs/signal-hook/*/signal_hook/fn.register.html)
    ///   apply.
    /// * This installs a signal handler. Signal handlers are program-global entities, so you may
    ///   be careful.
    /// * If there are multiple handles for the same signal, they share their signal handler ‒ only
    ///   the first one for each signal registers one.
    /// * Upon signal registration, the original handler is stored and called in chain from our own
    ///   signal handler.
    /// * A single handle can be used for multiple signals.
    /// * To unregister a handle from a signal handle, use the returned `SigId` and the
    ///   [`signal_hook::unregister`](https://docs.rs/signal-hook/*/signal_hook/fn.unregister.html).
    pub fn register_signal(&self, signal: libc::c_int) -> Result<SigId, Error> {
        signal_hook::flag::register(signal, Arc::clone(&self.0))
    }
}

#[cfg(all(test, not(windows)))] // Not testing on windows, very limited signal support
mod tests {
    use std::io::Read;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;
    use std::time::Duration;

    use super::*;

    struct Fake(Arc<AtomicUsize>);

    impl Read for Fake {
        fn read(&mut self, _buf: &mut [u8]) -> Result<usize, Error> {
            Ok(0) // Pretend it got closed… doesn't matter what we return here.
        }
    }

    #[test]
    fn signal_sent() {
        let opened_times = Arc::new(AtomicUsize::new(0));
        let opened_times_cp = Arc::clone(&opened_times);
        let mut reopen = crate::Reopen::new(Box::new(move || {
            opened_times_cp.fetch_add(1, Ordering::Relaxed);
            Ok(Fake(Arc::clone(&opened_times_cp)))
        }))
        .unwrap();
        assert_eq!(1, opened_times.load(Ordering::Relaxed));
        let mut buf = [0];
        assert_eq!(0, reopen.read(&mut buf).unwrap());
        assert_eq!(1, opened_times.load(Ordering::Relaxed));
        // Don't register sooner, in case some other test uses the signal.
        reopen.handle().register_signal(libc::SIGHUP).unwrap();
        // Now send us a signal
        unsafe { libc::kill(libc::getpid(), libc::SIGHUP) };
        // Wait a little for the signal to propagate, as it might arrive into another thread. The
        // second here is not guaranteed to work, this is only a hack for tests.
        thread::sleep(Duration::from_secs(1));
        assert_eq!(0, reopen.read(&mut buf).unwrap());
        // It got reopened
        assert_eq!(2, opened_times.load(Ordering::Relaxed));
    }
}
