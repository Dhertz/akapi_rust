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
use std::collections::{HashMap,HashSet};

use chrono::{Datelike,DateTime,Duration,Local,Timelike,Weekday};
use reqwest::Method::{Get, Post};
use lettre::email::EmailBuilder;
use lettre::transport::EmailTransport;
use lettre::transport::smtp::SmtpTransportBuilder;
use serde::Serialize;
use serde_json::Value;

mod jobs;
mod secrets;

use jobs::Job;

fn twilio_request<T: Serialize>(method: reqwest::Method, url_params: Option<String>, form_data: Option<&T>) -> Value {
    let tw_client = reqwest::Client::new().unwrap();
    let mut url = "https://api.twilio.com/2010-04-01/Accounts/".to_owned() + secrets::TW_ACC_ID + "/Messages.json";
    match url_params {
        Some(url_params) => url = url + "?" + &url_params,
        None => ()
    };
    println!("{}", url);
    let mut res = tw_client
        .request(method, &url)
        .basic_auth(secrets::TW_UID.to_owned(), Some(secrets::TW_KEY.to_owned()));

    match form_data {
        Some(form_data) => res = res.form(form_data),
        None => ()
    };
    let mut json_str = String::new();
    res.send().unwrap().read_to_string(&mut json_str).unwrap();
    let json_resp = serde_json::from_str(&json_str).unwrap();
    println!("{}", json_resp);
    json_resp
}

pub fn twilio_get(url_params: String) -> Value {
    let opt_url_params = match url_params.as_ref() {
        "" => None,
        _  => Some(url_params)
    };
    twilio_request(Get, opt_url_params, None::<&String>)
}

pub fn twilio_post<T: Serialize>(form_data: &T) -> Value {
    let opt_form_data = Some(form_data);
    twilio_request(Post, None, opt_form_data)
}

fn gen_subs_and_messages(mut subscribers: PurpleSubs, messages: Value) -> (PurpleSubs, HashMap<String, String>) {
    let last_id = subscribers.last_id();
    // Make a mutable copy of subscribers so we can add or remove from it if needed
    let mut mut_subs = subscribers;
    let mut messages_to_send = HashMap::new();

    for message in messages["messages"].as_array().unwrap() {
        let from_num = message["from"].as_str().unwrap();
        if message["sid"].as_str().unwrap() == last_id {
            break;
        }
       let response = match message["body"].as_str().unwrap() {
            "subscribe" | "start" => mut_subs.add(from_num),
            "stop" | "unsubscribe" | "no" => mut_subs.remove(from_num),
            _ => "Weird!".to_string()
        };
        messages_to_send.insert(from_num.to_string(), response.to_string());
    }
    mut_subs.set_last_id(messages["messages"][0]["sid"].as_str().unwrap().to_string());
    (mut_subs, messages_to_send)
}

pub fn manage_sms_subs() {
    let messages = twilio_get("To=".to_owned() + secrets::TW_NUMBER);
    let mut subscribers = PurpleSubs::new("subscribers.txt".to_string()).unwrap();
    let (mut_subs, messages_to_send) = gen_subs_and_messages(subscribers, messages);
    mut_subs.save();
    for (number, response) in messages_to_send {
        twilio_post(&[("To", number), ("MessagingServiceSid", secrets::TW_SID.to_owned()), ("Body", response)]);
    }
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
            self.subs.insert(sub);
            return "Welcome !".to_string()
        } else {
            return "You're already signed up!".to_string()
        }
    }
    fn remove(&mut self, subscriber: &str) -> String {
        if self.subs.contains(&subscriber.to_string()) {
            self.subs.remove(&subscriber.to_string());
            return "Sorry to see you go!".to_string();
        } else {
            return "You weren't even on the list!".to_string();
        }
    }
    fn save(&self) {
        //let json = serde_json::to_json(self.data);
        //write json
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

