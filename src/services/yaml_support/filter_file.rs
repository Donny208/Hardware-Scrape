use serde::{Serialize, Deserialize};

// struct to hold the entire rule file
#[derive(Debug, Serialize, Deserialize)]
pub struct Filter {
    pub keywords: Vec<String>,
}