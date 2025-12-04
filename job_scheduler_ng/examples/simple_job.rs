use core::time::Duration;
use job_scheduler_ng::{Job, JobScheduler};
use std::time::Instant;

fn main() {
    const WAIT_SECONDS: u64 = 40;

    let mut sched = JobScheduler::new();

    sched.add(Job::new("0/10 * * * * *".parse().unwrap(), || {
        log("I get executed every 10th second!");
    }));

    sched.add(Job::new("*/4 * * * * *".parse().unwrap(), || {
        log("I get executed every 4 seconds!");
    }));

    log(&format!("Run for about {WAIT_SECONDS} seconds!"));
    log("Starting loop");
    let start = Instant::now();
    loop {
        sched.tick();

        std::thread::sleep(Duration::from_millis(500));

        // Check if we have waited long enough
        if start.elapsed().as_secs() >= WAIT_SECONDS {
            break;
        }
    }
    log("Finished. Goodby!");
    std::process::exit(0);
}

fn log(msg: &str) {
    println!(
        "{:?} - {:?} - {msg}",
        chrono::Utc::now(),
        std::thread::current().id()
    );
}
