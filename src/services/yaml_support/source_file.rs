use serde::{Serialize, Deserialize};

// struct to hold the entire rule file
// This allows us to split the rules without caring about their contents.
#[derive(Debug, Serialize, Deserialize)]
pub struct SourceFile {
    //pub sources: Vec<serde_yaml::Mapping>,
    pub sources: Vec<SingleSource>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SingleSource {
    pub id: String,
    pub enabled: bool,
    pub url: String,
    pub accepted_flair: Vec<String>,
    pub have_regex: String,
    pub want_regex: String

}