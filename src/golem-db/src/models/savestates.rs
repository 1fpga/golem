use crate::schema;
use diesel::prelude::*;

#[derive(Clone, Debug, Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::savestates)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct SaveState {
    pub id: i32,

    /// The name of this core.
    pub name: Option<String>,

    /// The core this savestate is linked to.
    pub core_id: i32,

    /// The game this savestate is linked to.
    pub game_id: i32,

    /// The path to the savestate's image (.ss file).
    pub path: String,

    /// The path to the savestate's screenshot, if available.
    pub screenshot_path: Option<String>,

    /// Whether this savestate is a favorite.
    pub favorite: bool,

    /// When this was created.
    pub created_at: chrono::NaiveDateTime,

    /// The last time this savestate was loaded.
    pub last_played: Option<chrono::NaiveDateTime>,
}

impl SaveState {
    pub fn get(
        conn: &mut crate::Connection,
        id: i32,
    ) -> Result<Option<Self>, diesel::result::Error> {
        schema::savestates::table.find(id).first(conn).optional()
    }

    pub fn list_for_game(
        conn: &mut crate::Connection,
        id: i32,
    ) -> Result<Vec<Self>, diesel::result::Error> {
        use schema::savestates::dsl;
        schema::savestates::table
            .select(schema::savestates::all_columns)
            .filter(dsl::game_id.eq(id))
            .load(conn)
    }

    pub fn create(
        conn: &mut crate::Connection,
        core_id: i32,
        game_id: i32,
        path: String,
        screenshot_path: Option<String>,
    ) -> Result<Self, diesel::result::Error> {
        use schema::savestates::dsl;
        diesel::insert_into(schema::savestates::table)
            .values((
                dsl::core_id.eq(core_id),
                dsl::game_id.eq(game_id),
                dsl::path.eq(path),
                dsl::screenshot_path.eq(screenshot_path),
            ))
            .execute(conn)?;
        schema::savestates::table
            .select(schema::savestates::all_columns)
            .order(dsl::id.desc())
            .first(conn)
    }
}
