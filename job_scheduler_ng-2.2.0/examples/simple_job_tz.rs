use chrono::{Local, Timelike, SecondsFormat};
use core::time::Duration;
use job_scheduler_ng::{Job, JobScheduler};
use std::time::Instant;

fn main() {
    const WAIT_SECONDS: u64 = 40;

    let mut sched = JobScheduler::new();
    let local_tz = Local::now();
    sched.set_timezone(*local_tz.offset());

    let local_future = local_tz + chrono::Duration::seconds(18);
    let local_sec = local_future.second();
    let local_min = local_future.minute();
    let local_hour = local_future.hour();
    let local_sched = format!("{local_sec} {local_min} {local_hour} * * *");

    sched.add(Job::new(local_sched.parse().unwrap(), || {
        log(format!("I should get executed at {}!", local_future.to_rfc3339_opts(SecondsFormat::Secs, false)).as_str());
    }));

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
        Local::now(),
        std::thread::current().id()
    );
}
