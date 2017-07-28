extern crate chrono;
extern crate lettre;

use std::error::Error;
use std::thread::JoinHandle;

use chrono::{Datelike,DateTime,Duration,Local,Timelike,Weekday};
use lettre::email::EmailBuilder;
use lettre::transport::smtp::SMTP_PORT;
use lettre::transport::EmailTransport;
use lettre::transport::smtp::SmtpTransportBuilder;

mod jobs;
mod secrets;

use jobs::Job;

const EMAIL_BODY:&str = "We're back! Coming to you from a new, possibly shiny RUST implementation of the complex proprietary PurpleDaze™️  algorithm,\
 here is your regularly schedulded reminder:
 Remember to wear one of your finest purple garments tomorrow.";

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
    let is_purple_wed = (dt.weekday() == Weekday::Wed) && (3 < dt.day()) && (dt.day() < 11);
    let is_purple_thu = (dt.weekday() == Weekday::Thu) && (21 == dt.day()) && (dt.month() == Month::Nov as u32);
    let is_purple_fri = (dt.weekday() == Weekday::Fri) && ((dt.day() < 6) || (dt.day() > 12));
    is_purple_wed || is_purple_thu || is_purple_fri
}

fn email_if_purple_daze() -> Result<(), Box<Error>> {
    let now = Local::now();
    if (now.hour() == 17) && is_purple_daze(now + Duration::days(1)) {
        println!("Is purpledaze tomorrow");
        let email = EmailBuilder::new()
                            .to(secrets::PURPLE_EMAIL)
                            .from(secrets::MY_EMAIL)
                            .body(&EMAIL_BODY.replace("{{NUMBER}}", secrets::TW_NUMBER))
                            .subject("Purple Daze Incoming!")
                            .build()?;

        let mut mailer = SmtpTransportBuilder::new((secrets::SMTP_HOST, SMTP_PORT))?.build();
        mailer.send(email)?;
        println!("Purple Daze reminder sent");
    } else {
        println!("Is not purpledaze tomorrow");
    }
    Ok(())
}
pub fn run_purple_mailer(wait_time: u64) -> JoinHandle<()> {
    let sj = jobs::StandardJob::new(wait_time, email_if_purple_daze);
    sj.run()
}
