extern crate chrono;
extern crate lettre;

extern crate reqwest;

extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

use std::fs::File;
use std::io::BufReader;
use std::io::Read;

use std::error::Error;
use std::thread::JoinHandle;

use chrono::{Datelike,DateTime,Duration,Local,Timelike,Weekday};
use reqwest::header::{Headers, Authorization, Basic};
use lettre::email::EmailBuilder;
use lettre::transport::EmailTransport;
use lettre::transport::smtp::SmtpTransportBuilder;

mod jobs;
mod secrets;

use jobs::Job;

pub fn twilio_client() {
    let tw_client = reqwest::Client::new().unwrap();
    let url = "https://api.twilio.com/2010-04-01/Accounts/".to_owned() + secrets::TW_ACC_ID + "/Messages.json?To=" + secrets::TW_NUMBER;
    println!("{}", url);
    let mut res = tw_client.get(&url).header(
        Authorization(
            Basic {
                username: secrets::TW_SID.to_owned(),
                password: Some(secrets::TW_KEY.to_owned())
            }
        )
    ).send().unwrap();
    let mut buf = BufReader::new(res);
    let mut json_str = String::new();
    buf.read_to_string(&mut json_str).unwrap();
    println!("{}", json_str);
}

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

#[derive(Serialize, Deserialize)]
struct PurpleSubs {
    subs: Vec<String>,
    last_id: String
}

fn get_purple_subs() -> Result<PurpleSubs, Box<Error>>{
    let purple_file = File::open("subscribers.txt")?;
    let mut buf = BufReader::new(purple_file);
    let mut json_str = String::new();
    buf.read_to_string(&mut json_str)?;
    let decoded_json: PurpleSubs = serde_json::from_str(&json_str)?;
    Ok(decoded_json)
}

pub fn run_purple_mailer(wait_time: u64) -> JoinHandle<()> {
    let sj = jobs::StandardJob::new(wait_time, email_if_purple_daze);
    sj.run()
}

