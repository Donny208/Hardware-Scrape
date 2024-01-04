use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use log::info;
use reqwest::Client;
use roux::response::BasicThing;
use roux::submission::SubmissionData;
use roux::Subreddit;
use crate::services::yaml_support::source_file::SingleSource as SingleSource;
use crate::services::telegram::Telegram;
const BUFFER_SECONDS: f64 = 2.5;
const MINUTE_OFFSET: u64 = 60 * 3;

//todo revist this when you have learned about lifetimes and maybe swap String for &'str
pub struct Reddit {
    sources: Vec<SingleSource>,
    filters: Vec<String>
}

impl Reddit {
    pub async fn new(filters: Vec<String>, sources: Vec<SingleSource>) -> Reddit {
        Reddit {
            sources,
            filters
        }
    }

    pub async fn check_posts(&self) {
        info!("Checking posts within the last {} minute(s)", MINUTE_OFFSET/60);
        let time_check = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(n) => (n.as_secs()-(MINUTE_OFFSET)) as f64, //set to 1 minute before
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        };
        let client = get_client().await;
        let subreddit = Subreddit::new_oauth("hardwareswap", &client); //todo make this not hardcoded and use sources.yaml
        let latest = subreddit.latest(15, None).await; //adjust the 10 as seen
        match latest {
            Ok(submissions) => {
                // First we filter out any submissions that are older than now minus 1 minute
                let valid_submissions: Vec<_> = submissions.data.children.into_iter().filter(|s| submission_check(s, time_check)).collect();

                //Iterate over all valid submissions and look for values from self.filters
                for s in valid_submissions{
                    self.filter_check(&s).await;
                }
            }
            Err(err) => {
                println!("Failed to get submissions{}", err);
            }
        }
    }

    async fn filter_check(&self, submission: &BasicThing<SubmissionData>) {
        let mut title = &submission.data.title.to_ascii_lowercase();
        let mut text = &submission.data.selftext.to_ascii_lowercase();
        let url =  submission.data.url.as_ref().unwrap();

        //Iterate over all fiters and look for match
        for filter in &self.filters{
            //Look for title match
            if (title.contains(filter) || text.contains(filter)){
                info!("Filter match found, sending message now.");
                let msg = format!("__Filter Match: {filter}__\n[Deal here]({})", url);
                Telegram::send(msg).await;
                return
            }
        }
    }
}

fn submission_check(submission: &BasicThing<SubmissionData>, time: f64) -> bool {
    let time_check = submission.data.created_utc > time-BUFFER_SECONDS;
    let flair_check = {
        if let Some(flair) = &submission.data.link_flair_text {
            flair.eq("SELLING") //todo have this check the filterFile, hardcoded right now to hardwareswap
        } else {
            false
        }
    };
    return time_check && flair_check
}

async fn get_client() -> Client {
    let user_agent = env::var("REDDIT_USER_AGENT").expect("REDDIT_USER_AGENT Missing");
    let client_id = env::var("REDDIT_CLIENT_ID").expect("REDDIT_CLIENT_ID Missing");
    let client_secret = env::var("REDDIT_CLIENT_SECRET").expect("REDDIT_CLIENT_SECRET Missing");
    let username = env::var("REDDIT_USERNAME").expect("REDDIT_USERNAME Missing");
    let password = env::var("REDDIT_PASSWORD").expect("REDDIT_PASSWORD Missing");

    let client = roux::Reddit::new(user_agent.as_str(), client_id.as_str(), client_secret.as_str())
        .username(username.as_str())
        .password(password.as_str())
        .login()
        .await;

    let me = match client {
        Ok(me) => {
            me
        }
        Err(err) =>{
            panic!("Failed to log into reddit: {err}")
        }
    };

    me.client
}