pub use diesel;
use diesel::sqlite::Sqlite;

pub mod models;
pub mod schema;

pub use diesel::sqlite::SqliteConnection as Connection;

pub fn establish_connection(
    database_url: &str,
) -> Result<Connection, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let mut conn = diesel::Connection::establish(database_url)?;

    run_migrations(&mut conn)?;
    Ok(conn)
}

use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();
fn run_migrations(
    connection: &mut impl MigrationHarness<Sqlite>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    // This will run the necessary migrations.
    //
    // See the documentation for `MigrationHarness` for
    // all available methods.
    connection.run_pending_migrations(MIGRATIONS)?;

    Ok(())
}
