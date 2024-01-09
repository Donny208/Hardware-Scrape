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

//todo revist this when you have learned about lifetimes and maybe swap String for &'str
pub struct Reddit {
    sources: Vec<SingleSource>,
    filters: Vec<String>,
    refresh_rate: u32
}

impl Reddit {
    pub async fn new(filters: Vec<String>, sources: Vec<SingleSource>) -> Reddit {
        let refresh = env::var("REFRESH_RATE").expect("REFRESH_RATE Missing")
            .parse()
            .unwrap();
        println!("My frefresh rate {refresh}");
        Reddit {
            sources,
            filters,
            refresh_rate: refresh * 60 //Convert to minutes
        }
    }

    pub async fn check_posts(&self) {
        ///
        /// Iterate over all subreddits in source.yaml and get the most recent n posts and check for filter matches
        /// on valid submissions.
        ///
        info!("Checking posts within the last {} minute(s)", self.refresh_rate/60);
        let time_check = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(n) => (n.as_secs()-(self.refresh_rate)) as f64, //set to 1 minute before
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        };
        let client = get_client().await;
        for source in &self.sources {
            if source.enabled {
                info!("-> Checking {}", source.id);
                let subreddit = Subreddit::new_oauth(source.id.as_ref(), &client); //todo make this not hardcoded and use sources.yaml
                let latest = subreddit.latest(15, None).await; //adjust the 10 as seen
                match latest {
                    Ok(submissions) => {
                        // First we filter out any submissions that are older than now minus 1 minute
                        let valid_submissions: Vec<_> = submissions.data.children.into_iter()
                            .filter(|s| submission_check(s, time_check, &source))
                            .collect();

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
        }
    }

    async fn filter_check(&self, submission: &BasicThing<SubmissionData>) {
        ///
        /// Compares values in filter.yaml to a submission's title and selftext.
        /// If a match is found then it will send out a Telegram notification
        ///
        let title = &submission.data.title.to_ascii_lowercase();
        let text = &submission.data.selftext.to_ascii_lowercase();
        let url =  submission.data.url.as_ref().unwrap();

        //Iterate over all filters and look for match
        for filter in &self.filters{
            //Look for title match
            if (title.contains(filter) || text.contains(filter)){
                info!("Filter match found, sending message now.");
                let msg = format!("__Filter Match in {}: \
                    {filter}__\n[Deal here]({})",submission.data.subreddit, url
                );
                Telegram::send(msg).await;
                return
            }
        }
    }
}

fn submission_check(submission: &BasicThing<SubmissionData>, time: f64, source: &SingleSource) -> bool {
    ///
    /// Checks if a submission is within the current time scope and contains the correct flair per subreddit
    ///
    let time_check = submission.data.created_utc > time-BUFFER_SECONDS;
    let flair_check = {
        if let Some(flair) = &submission.data.link_flair_text {
            source.accepted_flair.iter().any(|f|f.eq(flair))
        } else {
            false
        }
    };
    return time_check && flair_check
}

async fn get_client() -> Client {
    ///
    /// Returns an authenticated reddit client
    ///
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