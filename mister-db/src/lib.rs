use dotenvy::dotenv;

pub use diesel;

pub mod models;
pub mod schema;

pub use diesel::sqlite::SqliteConnection as Connection;

pub fn establish_connection() -> Connection {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    diesel::Connection::establish(&database_url).expect("Error connecting to {database_url}")
}
