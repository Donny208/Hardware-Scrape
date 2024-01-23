use mongodb::{ bson::doc, options::{ ClientOptions, ServerApi, ServerApiVersion }, Client };
use mongodb::bson::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct MongoWrapper {
    connection_string: String,
    database: String,
    client: Client
}

impl MongoWrapper {
    pub async fn new(username: String, password: String, host: String, database: String) -> Result<MongoWrapper, mongodb::error::Error> {
        let connection_string = format!("mongodb+srv://{}:{}@{}", username,password,host);
        let client = MongoWrapper::get_client(&connection_string).await?;
        Ok(
            MongoWrapper {
                connection_string,
                database,
                client
            }
        )
    }

    pub async fn get_client(conn_string: &String) -> Result<Client, mongodb::error::Error> {
        let mut client_options = ClientOptions::parse(conn_string).await?;

        // Set the server_api field of the client_options object to Stable API version 1
        let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
        client_options.server_api = Some(server_api);

        // Create a new client and connect to the server
        let client = Client::with_options(client_options)?;

        Ok(client)
    }

    pub async fn health_check(&self) -> Result<(), mongodb::error::Error>{
        // Send a ping to confirm a successful connection
        self.client.database("admin").run_command(doc! { "ping": 1 }, None).await?;
        println!("Pinged your deployment. You successfully connected to MongoDB!");
        return Ok(())
    }

    pub async fn add_document(&self,
                        author: String,
                        title: String,
                        body: String,
                        submitted_utc: DateTime) -> Result<bool, mongodb::error::Error>{
        let deal_col = self.client.database("hardware_scrape").collection("deals");
        let retval = deal_col.insert_one(
            Deal {
                author,
                title,
                body,
                submitted_utc
            }, None).await?;
        println!("Inserted document with _id: {}", retval.inserted_id);
        Ok(true)
    }
}

#[derive(Serialize, Deserialize)]
struct Deal {
    author: String,
    title: String,
    body: String,
    submitted_utc: DateTime
}