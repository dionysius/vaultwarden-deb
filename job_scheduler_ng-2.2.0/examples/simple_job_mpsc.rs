use core::time::Duration;
use job_scheduler_ng::{Job, JobScheduler};
use std::sync::mpsc::{channel, Receiver, Sender};

fn main() {
    const WAIT_SECONDS: u64 = 40;

    let mut sched = JobScheduler::new();

    sched.add(Job::new("0/10 * * * * *".parse().unwrap(), || {
        log("I get executed every 10th second!");
    }));

    sched.add(Job::new("*/4 * * * * *".parse().unwrap(), || {
        log("I get executed every 4 seconds!");
    }));

    // Create a Send/Receive channel using a String
    let (tx, rx): (Sender<String>, Receiver<String>) = channel();

    // Create the receiver thread and print when we receive something
    std::thread::Builder::new()
        .name(String::from("channel-receiver"))
        .spawn(move || {
            log("Starting channel receiver loop within thread");
            loop {
                if let Ok(msg) = rx.recv() {
                    log(&format!("rx: {msg}"));
                }
                std::thread::sleep(Duration::from_millis(500));
            }
        })
        .expect("Error spawning channel-receiver thread");

    // Create a job which sends a message via the channel
    sched.add(Job::new("0/5 * * * * *".parse().unwrap(), {
        move || {
            tx.send(String::from(
                "I get executed every 5th second and send an mpsc!",
            ))
            .unwrap();
        }
    }));

    std::thread::Builder::new()
        .name(String::from("job-scheduler"))
        .spawn(move || {
            log("Starting job scheduler loop within thread");
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
