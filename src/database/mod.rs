pub mod users;
pub mod user_todo;

use mongodb::{options::ClientOptions, Client};

use self::{users::UserCollection, user_todo::UserTodo};

const MONGO_URL_DEFAULT: &str = "mongodb://localhost:27017";

pub struct DatabaseManager {
    pub users: UserCollection,
    pub todo: UserTodo,
}

impl DatabaseManager {
    pub async fn new() -> DatabaseManager {
        let url = std::env::var("MONGO_URL").unwrap_or(MONGO_URL_DEFAULT.to_string());
        println!("Connecting to mongodb at '{}'", url);
        let opt = ClientOptions::parse(&url).await.unwrap();
        let client = Client::with_options(opt).expect("Failed to start mongodb client");
        let db = client.database("TODO");

        DatabaseManager {
            users: UserCollection::new(&db),
            todo: UserTodo::new(&db)
        }
    }
}