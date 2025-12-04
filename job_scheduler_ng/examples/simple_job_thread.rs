use core::time::Duration;
use job_scheduler_ng::{Job, JobScheduler};

fn main() {
    const WAIT_SECONDS: u64 = 40;

    let mut sched = JobScheduler::new();

    sched.add(Job::new("0/10 * * * * *".parse().unwrap(), || {
        log("I get executed every 10th second!");
    }));

    sched.add(Job::new("*/4 * * * * *".parse().unwrap(), || {
        log("I get executed every 4 seconds!");
    }));

    std::thread::Builder::new()
        .name(String::from("job-scheduler"))
        .spawn(move || {
            log("Starting loop within thread");
            loop {
                sched.tick();
                std::thread::sleep(Duration::from_millis(500));
            }
        })
        .expect("Error spawning job-scheduler thread");

    log(&format!("Run for about {WAIT_SECONDS} seconds!"));
    std::thread::sleep(Duration::from_secs(WAIT_SECONDS));
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
