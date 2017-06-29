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
use std::io::Write;

use std::error::Error;
use std::thread::JoinHandle;
use std::collections::{HashMap,HashSet};

use chrono::{Datelike,DateTime,Duration,Local,Timelike,Weekday};
use lettre::email::EmailBuilder;
use lettre::transport::EmailTransport;
use lettre::transport::smtp::SmtpTransportBuilder;
use serde_json::Value;

mod jobs;
mod secrets;
mod twilio;

use jobs::Job;

const NEW_SUB_MSG:&str = "Thanks for subscribing to Puple Daze text updates. I'll be sure to let you know when to dress up!💃";
const OLD_SUB_MSG:&str = "Woah there eager beaver, looks like you are already on the VIP list! I'll make sure you get special treatment though.";
const UNSUB_MSG:&str = "Oh no!, I am sorry to see you go, but I will no longer remind you to encounter the Purple Daze 😔.";
const NOT_SUBD:&str = "Don't worry, you weren't even included yet. I'm not hurt, I didn't like the look of your phone number anyway.";
const WEIRD_MSG:&str = "Hmm. Not quite sure I know what you mean! 🤔 Respond with start to subscribe to notifications of Purple Daze or stop to unsubscribe.";

const EMAIL_BODY:&str = "We're back! Coming to you from a new, possibly shiny RUST implementation of the complex proprietary PurpleDaze™️  algorithm,\
 here is your regularly schedulded reminder:
 Remember to wearone of your finest purple garments tomorrow.

 Do you need an extra reminder tomorrow morning? I can send you a SMS! Sign up by texting START to {{NUMBER}}
 As it has been a while, please re-subscribe yourself to the SMS service if you wish to continue receiving alerts.";

fn gen_subs_and_messages(subscribers: PurpleSubs, messages: Value) -> Result<(PurpleSubs, HashMap<String, String>), Box<Error>> {
    let last_id = subscribers.last_id();
    // Make a mutable copy of subscribers so we can add or remove from it if needed
    let mut mut_subs = subscribers;
    let mut messages_to_send = HashMap::new();
    for message in twilio::option(messages["messages"].as_array())? {
        let from_num = twilio::option(message["from"].as_str())?;
        if twilio::option(message["sid"].as_str())? == last_id {
            break;
        }
        let response = match twilio::option(message["body"].as_str())?.to_lowercase().as_ref() {
            "subscribe" | "start" => mut_subs.add(from_num),
            "stop" | "unsubscribe" | "no" => mut_subs.remove(from_num),
            _ => WEIRD_MSG.to_string()
        };
        messages_to_send.insert(from_num.to_string(), response.to_string());
    }
    mut_subs.set_last_id(twilio::option(messages["messages"][0]["sid"].as_str())?.to_string());
    Ok((mut_subs, messages_to_send))
}

fn manage_sms_subs() -> Result<PurpleSubs, Box<Error>> {
    println!("Checking for new subscribers");
    let messages = twilio::get("To=".to_owned() + secrets::TW_NUMBER)?;
    let subscribers = PurpleSubs::new("subscribers.txt".to_string())?;
    let (mut_subs, messages_to_send) = gen_subs_and_messages(subscribers, messages)?;
    mut_subs.save("subscribers.txt".to_string())?;
    for (number, response) in messages_to_send {
        twilio::post(&[("To", number), ("MessagingServiceSid", secrets::TW_SID.to_string()), ("Body", response.to_string())])?;
    }
    Ok(mut_subs)
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

        let mut mailer = SmtpTransportBuilder::localhost()?.build();
        mailer.send(email)?;
        println!("Purple Daze reminder sent");
    } else {
        println!("Is not purpledaze tomorrow");
    }
    Ok(())
}

fn manage_purple_subs() -> Result<(), Box<Error>> {
    manage_sms_subs()?;
    Ok(())
}

fn text_if_purple_daze() -> Result<(), Box<Error>> {
    let subs = manage_sms_subs()?.subs;
    let now = Local::now();
    if (now.hour() == 7) && is_purple_daze(now) {
        println!("Is purpledaze today!");
        let reminder = "Remember it is Purple Daze today!".to_string();
        for sub in subs {
            twilio::post(&[("To", sub), ("MessagingServiceSid", secrets::TW_SID.to_string()), ("Body", reminder.to_string())])?;
        }
        println!("Purple Daze text sent");
    } else {
        println!("Is not purpledaze today");
    }
    Ok(())
}

#[derive(Serialize, Deserialize)]
struct PurpleSubs {
    subs: HashSet<String>,
    last_id: String
}

impl PurpleSubs {
    fn new(filename: String) -> Result<Self, Box<Error>> {
        let purple_file = File::open(filename)?;
        let mut buf = BufReader::new(purple_file);
        let mut json_str = String::new();
        buf.read_to_string(&mut json_str)?;
        Ok(serde_json::from_str(&json_str)?)
    }
    fn add(&mut self, subscriber: &str) -> String {
        let sub = subscriber.to_string();
        if !self.subs.contains(&sub) {
            println!("Adding {} to subs", sub);
            self.subs.insert(sub);
            return NEW_SUB_MSG.to_string()
        } else {
            return OLD_SUB_MSG.to_string()
        }
    }
    fn remove(&mut self, subscriber: &str) -> String {
        let sub = subscriber.to_string();
        if self.subs.contains(&sub) {
            println!("Removing {} from subs", sub);
            self.subs.remove(&sub);
            return UNSUB_MSG.to_string();
        } else {
            return NOT_SUBD.to_string();
        }
    }
    fn save(&self, filename: String) -> Result<(), Box<Error>>{
        let json = serde_json::to_string(&self)?;
        let mut f = File::create(filename)?;
        f.write_all(json.as_bytes())?;
        Ok(())
    }
    fn set_last_id(&mut self, last_id: String) {
        self.last_id = last_id;
    }
    fn last_id(&self) -> String {
        self.last_id.clone()
    }
}

pub fn run_purple_mailer(wait_time: u64) -> JoinHandle<()> {
    let sj = jobs::StandardJob::new(wait_time, email_if_purple_daze);
    sj.run()
}


pub fn run_purple_subs(wait_time: u64) -> JoinHandle<()> {
    let sj = jobs::StandardJob::new(wait_time, manage_purple_subs);
    sj.run()
}


pub fn run_purple_texter(wait_time: u64) -> JoinHandle<()> {
    let sj = jobs::StandardJob::new(wait_time, text_if_purple_daze);
    sj.run()
}
