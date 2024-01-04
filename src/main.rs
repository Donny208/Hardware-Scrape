mod services;
use services::yaml_support::filter_file::Filter as FilterFile;
use services::yaml_support::source_file::SourceFile;
use services::reddit::Reddit;
use dotenv::dotenv;
use std::fs;
use log::info;

#[tokio::main]
async fn main() {
    start_load();

    let (filter_file, source_file) = load_configs(false);
    let reddit = Reddit::new(filter_file.keywords, source_file.sources);

    reddit.await.check_posts().await;
}

//Basic inits for env
fn start_load() {
    dotenv().ok();
    env_logger::init();
}

fn load_configs(verbose: bool) -> (FilterFile, SourceFile){
    // Load the Filters
    let filter_contents = fs::read_to_string("./filter.yaml").expect("Should have been able to read the file");
    let filter_file: FilterFile = serde_yaml::from_str::<FilterFile>(&filter_contents).unwrap();

    //Load the Subreddit Sources
    let source_contents = fs::read_to_string("./source.yaml").expect("Should have been able to read the file");
    let source_file: SourceFile = serde_yaml::from_str(&source_contents).unwrap();

    if (verbose){
        println!("{:#?}", filter_file);
        println!("Sources Found: ");
        for source in &source_file.sources {
            println!("{}:\n{:#?}", source.id, source);
        }
    }
    return (filter_file, source_file)
}