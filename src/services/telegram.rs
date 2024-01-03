use std::collections::HashMap;
use std::env;
use log::{error, info};
use reqwest;

// todo revisit this class and the String datatype once you have learned lifetimes
pub struct Telegram {
    chat_id: String,
    token: String
}

impl Telegram {
    pub fn new (chat_id: String, token: String) -> Telegram {
        Telegram {
            chat_id,
            token
        }
    }
    pub async fn send(msg: String) -> () {
        let token = env::var("TELEGRAM_TOKEN").expect("TELEGRAM_TOKEN Missing");
        let chat_id = env::var("TELEGRAM_CHAT_ID").expect("TELEGRAM_CHAT_ID Missing");


        //Creating the client, url, and payload
        let client = reqwest::Client::new();
        let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
        let mut payload = HashMap::new();
        payload.insert("chat_id", chat_id);
        payload.insert("text", msg);
        payload.insert("parse_mode", String::from("MarkdownV2"));

        //Sending the request and processing ok & err, if ok check the status code for 200
        match client.post(&url)
            .json(&payload)
            .send()
            .await {
            Ok(res) => match res.status() {
                reqwest::StatusCode::OK => {
                    info!("Successfully sent message");
                }
                other_code => {
                    error!("Non-200 code returned: {other_code}");
                }
            }
            Err(error) => {
                error!("Failed to send msg: {:?}", error);
            }
        }
    }
}