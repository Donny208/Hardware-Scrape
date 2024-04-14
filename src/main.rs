mod services;
use services::yaml_support::filter_file::Filter as FilterFile;
use services::yaml_support::source_file::SourceFile;
use services::reddit::Reddit;
use dotenv::dotenv;
use std::{env, fs};
use mongodb::bson::DateTime;
use crate::services::db::MongoWrapper;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    load_helpers();


    let config = HSConfig::new(false);

    let mongo = MongoWrapper::new(
        config.mongo_username,
        config.mongo_password,
        config.mongo_host,
        config.mongo_database
    ).await?;

    let reddit = Reddit::new(config.filter_file.keywords, config.source_file.sources);

    reddit.await.check_posts(&mongo).await;

    Ok(())
}

//Basic inits for env
fn load_helpers() {
    dotenv().ok();
    env_logger::init();
}

struct HSConfig {
    filter_file: FilterFile,
    source_file: SourceFile,
    mongo_username: String,
    mongo_password: String,
    mongo_host: String,
    mongo_database: String
}

impl HSConfig {
    fn new (verbose: bool) -> HSConfig {
        // Load the Filters
        let filter_contents = fs::read_to_string("./filter.yaml").expect("Should have been able to read the file");
        let filter_file: FilterFile = serde_yaml::from_str::<FilterFile>(&filter_contents).unwrap();

        //Load the Subreddit Sources
        let source_contents = fs::read_to_string("./source.yaml").expect("Should have been able to read the file");
        let source_file: SourceFile = serde_yaml::from_str(&source_contents).unwrap();

        if verbose{
            println!("{:#?}", filter_file);
            println!("Sources Found: ");
            for source in &source_file.sources {
                println!("{}:\n{:#?}", source.id, source);
            }
        }

        return HSConfig {
            filter_file,
            source_file,
            mongo_username: env::var("MONGODB_USERNAME").expect("MONGODB_USERNAME Missing"),
            mongo_password: env::var("MONGODB_PASSWORD").expect("MONGODB_PASSWORD Missing"),
            mongo_host: env::var("MONGODB_HOST").expect("MONGODB_HOST Missing"),
            mongo_database: env::var("MONGODB_DATABASE").expect("MONGODB_DATABASE Missing")
        }
    }
}