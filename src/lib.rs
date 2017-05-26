extern crate chrono;
extern crate lettre;

use std::error::Error;
use std::thread::{JoinHandle,sleep,spawn};
use std::time::Duration as StdDuration;

use chrono::{Datelike,DateTime,Duration,Local,Timelike,Weekday};
use lettre::email::EmailBuilder;
use lettre::transport::EmailTransport;
use lettre::transport::smtp::SmtpTransportBuilder;

mod secrets;

#[allow(dead_code)]
enum Month {
    Jan =  1,
    Feb =  2,
    Mar =  3,
    Apr =  4,
    May =  5,
    Jun =  6,
    Jul =  7,
    Aug =  8,
    Sep =  9,
    Oct = 10,
    Nov = 11,
    Dec = 12,
}

fn is_purple_daze(dt: DateTime<Local>) -> bool {
    let is_purple_wed = (dt.weekday() == Weekday::Wed) & (3 < dt.day()) & (dt.day() < 11);
    let is_purple_thu = (dt.weekday() == Weekday::Thu) & (21 == dt.day()) & (dt.month() == Month::Nov as u32);
    let is_purple_fri = (dt.weekday() == Weekday::Fri) & (dt.day() < 6) | (dt.day() > 12);
    is_purple_wed | is_purple_thu | is_purple_fri
}

fn is_purple_daze_now() -> bool {
    is_purple_daze(Local::now())
}

fn email_if_purple_daze() -> Result<(), Box<Error>> {
    let now = Local::now();
    if true | (now.hour() == 17) & is_purple_daze(now + Duration::days(1)) {
        println!("Is purpledaze tomorrow");
        let email = EmailBuilder::new()
                            .to(secrets::TEST_EMAIL)
                            .from(secrets::MY_EMAIL)
                            .body("test")
                            .subject("Test")
                            .build()?;

        let mut mailer = SmtpTransportBuilder::localhost()?.build();
        mailer.send(email)?;
        println!("Purple Daze reminder sent");
    } else {
        println!("Is not purpledaze tomorrow");
    }
    Ok(())
}

trait Job {
    fn wait_time(&self) -> u64;
    fn thread_job(&self) -> fn() -> Result<(), Box<Error>>; 
    fn run(&self) -> JoinHandle<()>
    {
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


struct StandardJob {
    wait_time: u64,
    thread_job: fn() -> Result<(), Box<Error>>
}

impl StandardJob {
    fn new(wait_time: u64, thread_job: fn() -> Result<(), Box<Error>>) -> StandardJob {
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

pub fn run_purple_mailer(wait_time: u64) -> JoinHandle<()> {
    let sj = StandardJob::new(wait_time, email_if_purple_daze);
    sj.run()
}

