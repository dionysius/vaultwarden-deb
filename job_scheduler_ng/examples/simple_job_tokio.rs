use core::time::Duration;
use job_scheduler_ng::{Job, JobScheduler};
use std::hash::{DefaultHasher, Hash, Hasher};

#[tokio::main]
async fn main() {
    const WAIT_SECONDS: u64 = 40;

    log("Initializing scheduler!");
    init_scheduler();

    log(&format!("Run for about {WAIT_SECONDS} seconds!"));
    tokio::time::sleep(Duration::from_secs(WAIT_SECONDS)).await;
    log("Finished. Goodby!");
    std::process::exit(0);
}

fn init_scheduler() {
    // Start a new runtime to not mess with the current running one
    let runtime = tokio::runtime::Runtime::new().unwrap();

    std::thread::Builder::new()
        .name(String::from("job-scheduler"))
        .spawn(move || {
            let _runtime_guard = runtime.enter();

            let mut sched = JobScheduler::new();

            sched.add(Job::new("0/10 * * * * *".parse().unwrap(), || {
                log("I get executed every 10th second!");
            }));

            sched.add(Job::new("*/4 * * * * *".parse().unwrap(), || {
                log("I get executed every 4 seconds!");
            }));

            sched.add(Job::new("0/5 * * * * *".parse().unwrap(), || {
                runtime.spawn(test_job_every_five());
            }));

            sched.add(Job::new("*/8 * * * * *".parse().unwrap(), || {
                runtime.spawn(test_job_every_eight());
            }));

            log("Starting loop");
            loop {
                sched.tick();
                runtime.block_on(async move {
                    tokio::time::sleep(Duration::from_millis(500)).await;
                });
            }
        })
        .expect("Error spawning job-scheduler thread");
}

async fn test_job_every_five() {
    let hash = hash(&chrono::Utc::now().timestamp_millis());
    log(&format!(
        "I get executed every 5th second (begin - {hash})!"
    ));
    // Wait 7 seconds, this will demonstrate this call will be async and not blocking.
    tokio::time::sleep(Duration::from_secs(7)).await;
    log(&format!(
        "I get executed every 5th second (end   - {hash})!"
    ));
}

async fn test_job_every_eight() {
    tokio::time::sleep(Duration::from_millis(500)).await;
    log("I get executed every 8 seconds!");
}

fn log(msg: &str) {
    println!(
        "{:?} - {:?} - {msg}",
        chrono::Utc::now(),
        std::thread::current().id()
    );
}

fn hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
