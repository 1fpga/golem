use boa_engine::{js_error, JsError, JsResult};
use include_dir::{include_dir, Dir, File};
use sqlite::State;
use std::borrow::Cow;
use std::ffi::OsStr;
use std::path::PathBuf;
use tracing::debug;

static MIGRATION_DIR: Dir<'_> = include_dir!("$OUT_DIR/migrations");

fn get_latest_migration(connection: &sqlite::Connection) -> Option<String> {
    let query = "SELECT value FROM _golem_settings WHERE key = 'migration'";

    let mut statement = connection.prepare(query).ok()?;

    if let Ok(State::Row) = statement.next() {
        if let Ok(value) = statement.read::<String, _>("value") {
            return Some(value);
        }
    }

    None
}

fn list_all_migrations(name: &str) -> Vec<String> {
    MIGRATION_DIR
        .get_dir(name)
        .into_iter()
        .flat_map(|dir| {
            dir.dirs()
                .map(|d| d.path().file_name().unwrap_or_default().to_string_lossy())
        })
        .map(Cow::into_owned)
        .collect()
}

fn list_up_migrations(name: &str, latest_migration: Option<&str>) -> Vec<(String, File<'static>)> {
    list_all_migrations(name)
        .into_iter()
        .filter(|m| {
            if let Some(lm) = latest_migration {
                m.as_str() > lm
            } else {
                true
            }
        })
        .filter_map(|m| {
            MIGRATION_DIR
                .get_file(&format!("{}/{}/up.sql", name, m))
                .map(|f| (m, f.clone()))
        })
        .collect()
}

fn apply_migrations_from(
    connection: &sqlite::Connection,
    name: &str,
    latest_migration: Option<&str>,
) -> JsResult<()> {
    let migrations = list_up_migrations(name, latest_migration);
    debug!(?migrations, "Applying migrations");

    for (name, migration) in migrations {
        debug!(name, "Applying migration");

        let query = migration.contents_utf8().unwrap();
        connection.execute(query).map_err(JsError::from_rust)?;

        let mut statement = connection
            .prepare(
                "INSERT INTO _golem_settings (key, value) VALUES ('migration', ?)\
                    ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            )
            .map_err(JsError::from_rust)?;

        statement
            .bind((1, name.as_str()))
            .map_err(JsError::from_rust)?;

        if let Ok(State::Done) = statement.next() {
            debug!(name, "Migration applied");
        } else {
            return Err(js_error!("Failed to apply migration"));
        }
    }

    Ok(())
}

fn initialize(connection: &sqlite::Connection, name: &str) -> JsResult<()> {
    debug!(name, "Initializing database");
    connection
        .execute(
            "\
            CREATE TABLE IF NOT EXISTS _golem_settings \
            (key TEXT PRIMARY KEY, value TEXT)\
        ",
        )
        .map_err(JsError::from_rust)?;

    apply_migrations_from(connection, name, None)?;

    Ok(())
}

pub(super) fn apply_migrations(connection: &sqlite::Connection, name: &str) -> JsResult<()> {
    let latest_migration = get_latest_migration(connection);

    debug!(name, "Latest migration: {:?}", latest_migration);

    if let Some(latest_migration) = latest_migration {
        apply_migrations_from(connection, name, Some(&latest_migration))?;
    } else {
        initialize(connection, name)?;
    }

    Ok(())
}

#[test]
fn test_list_all_migrations() {
    let migrations = list_all_migrations("1fpga");
    assert!(migrations.contains(&"0000-00-00-000000_initial".to_string()));
}

#[test]
fn test_list_up_migrations_empty() {
    let migrations = list_up_migrations("1fpga", None);
    assert!(!migrations.is_empty());
    assert!(migrations
        .iter()
        .any(|(m, _)| m.as_str() == "0000-00-00-000000_initial"));
}

#[test]
fn test_list_up_migrations() {
    let migrations = list_up_migrations("1fpga", Some("0000-00-00-000000_initial"));
    assert!(!migrations
        .iter()
        .any(|(m, _)| m.as_str() == "0000-00-00-000000_initial"));
}
