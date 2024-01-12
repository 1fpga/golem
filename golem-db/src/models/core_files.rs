use crate::schema;
use diesel::prelude::*;

#[derive(Clone, Debug, Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::core_files)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct CoreFile {
    pub id: i32,

    /// The name of this core.
    pub name: Option<String>,

    /// The core this file is linked to.
    pub core_id: i32,

    /// The core index this file is linked to.
    pub core_index: i32,

    /// The game this file is linked to.
    pub game_id: i32,

    /// The path to the file's image (normally .sav file but depends on core).
    pub path: String,

    /// When this was created.
    pub created_at: chrono::NaiveDateTime,

    /// The last time this savestate was loaded.
    pub last_loaded: Option<chrono::NaiveDateTime>,
}

impl CoreFile {
    pub fn get(
        conn: &mut crate::Connection,
        id: i32,
    ) -> Result<Option<Self>, diesel::result::Error> {
        schema::core_files::table.find(id).first(conn).optional()
    }

    pub fn latest_for_game(
        conn: &mut crate::Connection,
        game_id: i32,
    ) -> Result<Option<Self>, diesel::result::Error> {
        use schema::core_files::dsl;
        schema::core_files::table
            .select(schema::core_files::all_columns)
            .filter(dsl::game_id.eq(game_id))
            .order(dsl::id.desc())
            .first(conn)
            .optional()
    }

    pub fn create(
        conn: &mut crate::Connection,
        name: Option<String>,
        core_id: i32,
        core_index: i32,
        game_id: i32,
        path: String,
    ) -> Result<Self, diesel::result::Error> {
        use schema::core_files::dsl;
        diesel::insert_into(schema::core_files::table)
            .values((
                dsl::name.eq(name),
                dsl::core_id.eq(core_id),
                dsl::core_index.eq(core_index),
                dsl::game_id.eq(game_id),
                dsl::path.eq(path),
                dsl::created_at.eq(chrono::Utc::now().naive_utc()),
            ))
            .execute(conn)?;
        schema::core_files::table
            .select(schema::core_files::all_columns)
            .order(dsl::id.desc())
            .first(conn)
    }

    pub fn load(&self, conn: &mut crate::Connection) -> Result<(), diesel::result::Error> {
        use schema::core_files::dsl;
        diesel::update(schema::core_files::table.find(self.id))
            .set(dsl::last_loaded.eq(chrono::Utc::now().naive_utc()))
            .execute(conn)?;
        Ok(())
    }
}
