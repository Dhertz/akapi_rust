use std::error::Error;
use std::io::Read;

use reqwest;
use reqwest::Method::{Get, Post};
use serde_json;
use serde::Serialize;
use serde_json::Value;

use secrets;

fn request<T: Serialize>(method: reqwest::Method, url_params: Option<String>, form_data: Option<&T>) -> Result<Value, Box<Error>> {
    let tw_client = reqwest::Client::new()?;
    let mut url = "https://api.twilio.com/2010-04-01/Accounts/".to_owned() + secrets::TW_ACC_ID + "/Messages.json";
    if let Some(url_params) = url_params {
        url = url + "?" + &url_params;
    };
    let mut res = tw_client
        .request(method, &url)
        .basic_auth(secrets::TW_UID.to_owned(), Some(secrets::TW_KEY.to_owned()));

    if let Some(form_data) = form_data {
        res = res.form(form_data);
    };
    let mut json_str = String::new();
    res.send()?.read_to_string(&mut json_str)?;
    let json_resp = serde_json::from_str(&json_str)?;
    Ok(json_resp)
}

pub fn get(url_params: String) -> Result<Value, Box<Error>> {
    let opt_url_params = match url_params.as_ref() {
        "" => None,
        _  => Some(url_params)
    };
    request(Get, opt_url_params, None::<&String>)
}

pub fn post<T: Serialize>(form_data: &T) -> Result<Value, Box<Error>> {
    let opt_form_data = Some(form_data);
    request(Post, None, opt_form_data)
}

pub fn option<T>(opt: Option<T>) -> Result<T, String> {
    match opt {
        Some(opt) => Ok(opt),
        None => Err("Weird Twilio JSON".to_string())
    }
}

