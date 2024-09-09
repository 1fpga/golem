pub use diesel;
use diesel::sqlite::Sqlite;
use diesel::{sql_query, RunQueryDsl};

pub use diesel::sqlite::SqliteConnection as Connection;

pub fn establish_connection(
    database_url: &str,
) -> Result<Connection, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let mut conn = diesel::Connection::establish(database_url)?;

    // Make sure we're in WAL mode.
    sql_query("PRAGMA journal_mode=WAL;").execute(&mut conn)?;

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
