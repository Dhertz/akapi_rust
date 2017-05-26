use std::error::Error;
use std::thread::{JoinHandle,sleep,spawn};
use std::time::Duration as StdDuration;

pub trait Job {
    fn wait_time(&self) -> u64;
    fn thread_job(&self) -> fn() -> Result<(), Box<Error>>;
    fn run(&self) -> JoinHandle<()> {
        let wait_time = self.wait_time();
        let thread_job = self.thread_job();
        let j = spawn(move || {
            loop {
                match thread_job() {
                    Ok(_) => (),
                    Err(err) => println!("Thread crashed: {}", err)
                };
                sleep(StdDuration::from_secs(wait_time));
            }
        });
        return j;
    }
}

pub struct StandardJob {
    wait_time: u64,
    thread_job: fn() -> Result<(), Box<Error>>
}

impl StandardJob {
    pub fn new(wait_time: u64, thread_job: fn() -> Result<(), Box<Error>>) -> StandardJob {
        StandardJob {
            wait_time: wait_time,
            thread_job: thread_job
        }
    }
}

impl Job for StandardJob {
    fn wait_time(&self) -> u64 {
        self.wait_time
    }
    fn thread_job(&self) -> fn() -> Result<(), Box<Error>> {
        self.thread_job
    }
}
