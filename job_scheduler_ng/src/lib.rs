//! # JobScheduler
//!
//! A simple cron-like job scheduling library for Rust.
//!
//! ## Usage
//!
//! Be sure to add the job_scheduler_ng crate to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! job_scheduler_ng = "*"
//! ```
//!
//! Creating a schedule for a job is done using the `FromStr` impl for the
//! `Schedule` type of the [cron](https://github.com/zslayton/cron) library.
//!
//! The scheduling format is as follows:
//!
//! ```text
//! sec   min   hour   day of month   month   day of week   year
//! *     *     *      *              *       *             *
//! ```
//!
//! Note that the year may be omitted.
//!
//! Comma separated values such as `5,8,10` represent more than one time
//! value. So for example, a schedule of `0 2,14,26 * * * *` would execute
//! on the 2nd, 14th, and 26th minute of every hour.
//!
//! Ranges can be specified with a dash. A schedule of `0 0 * 5-10 * *`
//! would execute once per hour but only on day 5 through 10 of the month.
//!
//! Day of the week can be specified as an abbreviation or the full name.
//! A schedule of `0 0 6 * * Sun,Sat` would execute at 6am on Sunday and
//! Saturday.
//!
//! A simple usage example:
//!
//! ```rust,ignore
//! use job_scheduler_ng::{JobScheduler, Job};
//! use core::time::Duration;
//!
//! fn main() {
//!     let mut sched = JobScheduler::new();
//!
//!     sched.add(Job::new("0/10 * * * * *".parse().unwrap(), || {
//!         println!("I get executed every 10th second!");
//!     }));
//!
//!     sched.add(Job::new("*/4 * * * * *".parse().unwrap(), || {
//!         println!("I get executed every 4 seconds!");
//!     }));
//!
//!     loop {
//!         sched.tick();
//!         std::thread::sleep(Duration::from_millis(500));
//!     }
//! }
//! ```
//!
//! Setting a custom timezone other then the default UTC
//! Any `Tz::Offset` provided by chrono will work.
//!
//! //! ```rust,ignore
//! use chrono::Local;
//! use job_scheduler_ng::{JobScheduler, Job};
//! use core::time::Duration;
//!
//! fn main() {
//!     let mut sched = JobScheduler::new();
//!     let local_tz = chrono::Local::now();
//!     sched.set_timezone(*local_tz.offset());
//!
//!     sched.add(Job::new("0 5 13 * * *".parse().unwrap(), || {
//!         println!("I get executed every day 13:05 local time!");
//!     }));
//!
//!     loop {
//!         sched.tick();
//!         std::thread::sleep(Duration::from_millis(500));
//!     }
//! }
//! ```

use chrono::{DateTime, Duration, FixedOffset, Utc};
pub use cron::Schedule;
pub use uuid::Uuid;

/// A schedulable `Job`.
pub struct Job<'a> {
    schedule: Schedule,
    run: Box<dyn (FnMut()) + Send + 'a>,
    last_tick: Option<DateTime<FixedOffset>>,
    limit_missed_runs: usize,
    job_id: Uuid,
    tz: FixedOffset,
}

impl<'a> Job<'a> {
    /// Create a new job.
    ///
    /// ```rust,ignore
    /// // Run at second 0 of the 15th minute of the 6th, 8th, and 10th hour
    /// // of any day in March and June that is a Friday of the year 2017.
    /// let s: Schedule = "0 15 6,8,10 * Mar,Jun Fri 2017".into().unwrap();
    /// Job::new(s, || println!("I have a complex schedule...") );
    /// ```
    #[inline]
    pub fn new<T>(schedule: Schedule, run: T) -> Self
    where
        T: FnMut() + Send + 'a,
    {
        Self {
            schedule,
            run: Box::new(run),
            last_tick: None,
            limit_missed_runs: 1,
            job_id: Uuid::new_v4(),
            tz: FixedOffset::east_opt(0).unwrap(),
        }
    }

    fn tick(&mut self) {
        let now = Utc::now().with_timezone(&self.tz);
        if let Some(last_tick) = self.last_tick {
            if self.limit_missed_runs > 0 {
                for event in self.schedule.after(&last_tick).take(self.limit_missed_runs) {
                    if event > now {
                        break;
                    }
                    (self.run)();
                }
            } else {
                for event in self.schedule.after(&last_tick) {
                    if event > now {
                        break;
                    }
                    (self.run)();
                }
            }
        }

        self.last_tick = Some(now);
    }

    /// Set the limit for missed jobs in the case of delayed runs. Setting to 0 means unlimited.
    ///
    /// ```rust,ignore
    /// let mut job = Job::new("0/1 * * * * *".parse().unwrap(), || {
    ///     println!("I get executed every 1 seconds!");
    /// });
    /// job.limit_missed_runs(99);
    /// ```
    #[inline]
    pub fn limit_missed_runs(&mut self, limit: usize) {
        self.limit_missed_runs = limit;
    }

    /// Set last tick to force re-running of missed runs.
    ///
    /// ```rust,ignore
    /// let mut job = Job::new("0/1 * * * * *".parse().unwrap(), || {
    ///     println!("I get executed every 1 seconds!");
    /// });
    /// job.last_tick(Some(Utc::now()));
    /// ```
    #[inline]
    pub fn last_tick(&mut self, last_tick: Option<DateTime<FixedOffset>>) {
        self.last_tick = last_tick;
    }
}

/// The JobScheduler contains and executes the scheduled jobs.
pub struct JobScheduler<'a> {
    jobs: Vec<Job<'a>>,
    tz: FixedOffset,
}

impl Default for JobScheduler<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> JobScheduler<'a> {
    /// Create a new `JobScheduler`.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            jobs: Vec::new(),
            tz: FixedOffset::east_opt(0).unwrap(),
        }
    }

    /// Set a custom timezone other then the default UTC.
    ///
    /// ```rust,ignore
    /// let mut sched = JobScheduler::new();
    /// let local_tz = chrono::Local::now();
    /// sched.set_timezone(*local_tz.offset());
    /// sched.add(Job::new("0 5 13 * * *".parse().unwrap(), || {
    ///     println!("I get executed every day 13:05 local time!");
    /// }));
    /// ```
    pub fn set_timezone(&mut self, tz: FixedOffset) {
        self.tz = tz;
    }

    /// Add a job to the `JobScheduler`
    ///
    /// ```rust,ignore
    /// let mut sched = JobScheduler::new();
    /// sched.add(Job::new("1/10 * * * * *".parse().unwrap(), || {
    ///     println!("I get executed every 10 seconds!");
    /// }));
    /// ```
    #[inline]
    pub fn add(&mut self, job: Job<'a>) -> Uuid {
        let job_id = job.job_id;

        let mut job = job;
        job.tz = self.tz;
        self.jobs.push(job);

        job_id
    }

    /// Remove a job from the `JobScheduler`
    ///
    /// ```rust,ignore
    /// let mut sched = JobScheduler::new();
    /// let job_id = sched.add(Job::new("1/10 * * * * *".parse().unwrap(), || {
    ///     println!("I get executed every 10 seconds!");
    /// }));
    /// sched.remove(job_id);
    /// ```
    #[inline]
    pub fn remove(&mut self, job_id: Uuid) -> bool {
        let mut found_index = None;
        for (i, job) in self.jobs.iter().enumerate() {
            if job.job_id == job_id {
                found_index = Some(i);
                break;
            }
        }

        if let Some(index) = found_index {
            self.jobs.remove(index);
        }

        found_index.is_some()
    }

    /// The `tick` method increments time for the JobScheduler and executes
    /// any pending jobs. It is recommended to sleep for at least 500
    /// milliseconds between invocations of this method.
    ///
    /// ```rust,ignore
    /// loop {
    ///     sched.tick();
    ///     std::thread::sleep(Duration::from_millis(500));
    /// }
    /// ```
    #[inline]
    pub fn tick(&mut self) {
        for job in &mut self.jobs {
            job.tick();
        }
    }

    /// The `time_till_next_job` method returns the duration till the next job
    /// is supposed to run. This can be used to sleep until then without waking
    /// up at a fixed interval.AsMut
    ///
    /// ```rust, ignore
    /// loop {
    ///     sched.tick();
    ///     std::thread::sleep(sched.time_till_next_job());
    /// }
    /// ```
    #[inline]
    pub fn time_till_next_job(&self) -> core::time::Duration {
        if self.jobs.is_empty() {
            // Take a guess if there are no jobs.
            return core::time::Duration::from_millis(500);
        }
        let tz = self.tz;
        let mut duration = Duration::zero();
        let now = Utc::now().with_timezone(&tz);
        for job in &self.jobs {
            for event in job.schedule.upcoming(tz).take(1) {
                let d = event - now;
                if duration.is_zero() || d < duration {
                    duration = d;
                }
            }
        }
        duration.to_std().unwrap()
    }
}
