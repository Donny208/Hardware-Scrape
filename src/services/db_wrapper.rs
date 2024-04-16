use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};
use sqlx::FromRow;
use chrono::{DateTime, LocalResult, TimeZone, Utc};
use roux::submission::SubmissionData;
use regex::Regex;
use std::str::FromStr;
use std::error::Error;
use std::fmt::Debug;
use std::io::{self, ErrorKind};

/////////////
// Schemas //
/////////////

#[derive(Debug, FromRow)]
pub struct RawPost {
    pub external_reddit_id: String,
    pub post_url: String,
    pub user_id: i32,
    pub post_time: Option<DateTime<Utc>>,
    pub country: String,
    pub state: String,
    pub have_text: String,
    pub want_text: String,
    pub is_selling: bool,
    pub body: String,
}

#[derive(Debug, FromRow)]
pub struct User {
    pub id: i32,
    pub user_name: String,
    pub trades: i32,
}


////////////////
// DB Handler //
////////////////

pub struct DatabaseHandler {
    conn_string: String,
    pool: Option<PgPool>,
}

impl DatabaseHandler {
    pub fn new(username: String, password: String, host: String, db_name: String) -> Self {
        let conn_string = format!("postgres://{username}:{password}@{host}/{db_name}");
        DatabaseHandler {
            conn_string,
            pool: None,
        }
    }

    pub async fn init_pool(&mut self) -> Result<(), sqlx::Error> {
        let pool = PgPoolOptions::new()
            .connect(&self.conn_string)
            .await?;
        self.pool = Some(pool);
        Ok(())
    }

    ///
    /// Checks if a user exists, if not will create a new user
    /// Will also update the user's trade count if the user exists
    /// Returns a user's ID
    ///
    pub async fn user_check(&self, submission_data: &SubmissionData) -> Result<i32, sqlx::Error> {
        let user_name = &submission_data.author;
        let trades = {
            if let Some(trade_text) = &submission_data.author_flair_text {
                user_trades_length(trade_text)
            } else{
                -1
            }
        };
        if let Some(pool) = &self.pool {
            // First, try to fetch the user by username to see if they exist
            let maybe_user = sqlx::query_as::<_, User>(
                "SELECT id, user_name, trades FROM users WHERE user_name = $1"
            )
                .bind(user_name)
                .fetch_optional(pool)
                .await?;

            match maybe_user {
                Some(existing_user) => {
                    // If user exists, update their trades and return existing id
                    sqlx::query("UPDATE users SET trades = $1 WHERE id = $2")
                        .bind(trades)
                        .bind(existing_user.id)
                        .execute(pool)
                        .await?;
                    Ok(existing_user.id)
                },
                None => {
                    // If user does not exist, insert and return new id
                    let user_id = sqlx::query("INSERT INTO users (user_name, trades) VALUES ($1, $2) RETURNING id")
                        .bind(user_name)
                        .bind(trades)
                        .fetch_one(pool) // fetch_one used to get the single returned id
                        .await?
                        .get(0);
                    println!("Added {}({}) to the users table", user_name, trades);
                    Ok(user_id)
                }
            }
        } else {
            // Handle the case where there is no pool initialized
            Err(sqlx::Error::PoolClosed)
        }
    }

    pub async fn does_post_exist(&self, post_id: &String) -> Result<bool, sqlx::Error> {
        if let Some(pool) = &self.pool {
            // Checking only for the existence of the post_id in the posts table
            let result = sqlx::query("SELECT EXISTS (SELECT 1 FROM raw_posts WHERE external_reddit_id = $1)")
                .bind(post_id)
                .fetch_one(pool)
                .await?;

            // Extract the boolean directly from the result row
            let exists = result.try_get::<bool, _>(0)?;
            Ok(exists)
        } else {
            // Handle the case where there is no pool initialized
            Err(sqlx::Error::PoolClosed)
        }
    }

    pub async fn add_document_from_submission(&self, submission_data: &SubmissionData) -> Result<(), Box<dyn Error>> {
        // First make sure the post doesn't already exist
        match self.does_post_exist(&submission_data.id).await {
            Ok(post_exists) => {
                if post_exists {
                    return Err(Box::new(io::Error::new(ErrorKind::InvalidInput, "This post already exists in the db")))
                }
            },
            Err(e) => return Err(Box::try_from(e).unwrap()),
        };

        // Then we validate the title
        let title_components = match post_validation(&submission_data.title) {
            Ok(components) => components,
            Err(e) => return Err(e),
        };

        // Then we look up the user
        let user_id = match self.user_check(&submission_data).await {
            Ok(id) => id,
            Err(e) => return Err(Box::new(e)),
        };

        // Then we format the submission into a db post row
        let post = match submission_to_raw_post(submission_data, title_components, user_id) {
            Ok(p) => p,
            Err(e) => return Err(e),
        };

        // Check if pool exists and then perform the SQL operation
        if let Some(pool) = &self.pool {
            let sql = "INSERT INTO raw_posts (external_reddit_id, post_url, user_id, post_time, country, state, have_text, want_text, is_selling, body)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)";
            sqlx::query(sql)
                .bind(&post.external_reddit_id)
                .bind(&post.post_url)
                .bind(post.user_id)
                .bind(post.post_time)
                .bind(&post.country)
                .bind(&post.state)
                .bind(&post.have_text)
                .bind(&post.want_text)
                .bind(post.is_selling)
                .bind(&post.body)
                .execute(pool)
                .await
                .map_err(|e| Box::new(e) as Box<dyn Error>)?;
            Ok(())
        } else {
            // Handle the case where there is no pool initialized
            Err(Box::new(sqlx::Error::PoolClosed))
        }
    }
}


struct ValidTitleComponents {
    country: String,
    state: String,
    have_text: String,
    want_text: String
}

fn submission_to_raw_post(submission_data: &SubmissionData, title_components: ValidTitleComponents, user_id: i32) -> Result<RawPost, Box<dyn Error>> {
    let is_selling = {
        match &submission_data.link_flair_text {
            Some(flair) => {
                flair == "SELLING"
            }
            None => {
                false
            }
        }
    };

    let timestamp = {
        match Utc.timestamp_opt(submission_data.created_utc as i64, 0) {
            LocalResult::Single(dt) => Some(dt),
            _ => None, // Handle the case where the timestamp is invalid
        }
    };

    let raw_post = RawPost {
        external_reddit_id: submission_data.id.clone(),
        post_url: submission_data.permalink.clone(),
        user_id,
        post_time: timestamp,
        country: title_components.country,
        state: title_components.state,
        have_text: title_components.have_text,
        want_text: title_components.want_text,
        is_selling,
        body: submission_data.selftext.clone(),
    };
    return Ok(raw_post);
}



///
/// Validates a submission's title, if not valid we don't add to the database
/// Returns have text, want text, country, and state
///
fn post_validation(post_title: &str) -> Result<ValidTitleComponents, Box<dyn Error>> {
    let re = Regex::new(r"\[(?<country>usa|USA)\s*-\s*(?<state>[a-zA-Z]{2}|nyc|NYC)\]\s*\[H\]\s*(?<have_text>.*)\s*\[W\]\s*(?<want_text>.*)").unwrap();
    match re.captures(post_title) {
        Some(captures) => {
            Ok(ValidTitleComponents {
                country: captures.name("country").unwrap().as_str().to_string(),
                state: captures.name("state").unwrap().as_str().to_string(),
                have_text: captures.name("have_text").unwrap().as_str().to_string(),
                want_text: captures.name("want_text").unwrap().as_str().to_string(),
            })
        },
        None => Err(Box::new(io::Error::new(ErrorKind::InvalidInput, "Invalid post title format")))
    }
}


///
/// Gets the users number of trades, if none sets it to -1
///
fn user_trades_length(user_flair: &String) -> i32{
    let re = Regex::new(r"Trades:\s*(?<trades>\d+)").unwrap();
    let Some(captures) = re.captures(user_flair) else {
        return -1
    };
    let retval: i32 = FromStr::from_str(&captures["trades"]).unwrap();
    return retval
}