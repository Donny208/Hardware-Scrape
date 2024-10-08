mod services;
use services::yaml_support::filter_file::Filter as FilterFile;
use services::yaml_support::source_file::SourceFile;
use services::reddit::Reddit;
use dotenv::dotenv;
use std::{env, fs};
use crate::services::db_wrapper::DatabaseHandler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    load_helpers();


    let config = HSConfig::new(false);

    let mut db = DatabaseHandler::new(
        config.db_username,
        config.db_password,
        config.db_host,
        config.db_database
    );
    
    db.init_pool().await.expect("Failed to init db");


    let reddit = Reddit::new(config.filter_file.keywords, config.source_file.sources);

    reddit.await.check_posts(&db).await;

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
    db_username: String,
    db_password: String,
    db_host: String,
    db_database: String
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
            db_username: env::var("POSTGRES_USERNAME").expect("DB_USERNAME Missing"),
            db_password: env::var("POSTGRES_PASSWORD").expect("DB_PASSWORD Missing"),
            db_host: env::var("POSTGRES_HOST").expect("DB_HOST Missing"),
            db_database: env::var("POSTGRES_DATABASE").expect("DB_DATABASE Missing")
        }
    }
}