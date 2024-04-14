use serde::{Serialize, Deserialize};

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
    pub want_regex: String,
    pub grab_amount: u32,
    pub save_to_db: bool
}