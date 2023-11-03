use crate::models;
use crate::schema;
use diesel::prelude::*;

#[derive(Debug, Queryable, Selectable, Identifiable)]
#[diesel(table_name = schema::dat_files)]
#[diesel(belongs_to(Core))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DatFile {
    pub id: i32,

    pub name: String,

    pub path: String,

    pub core_id: i32,

    pub priority: i32,
}

impl DatFile {
    pub fn create(
        conn: &mut crate::Connection,
        name: &str,
        path: &str,
        core: &models::Core,
        priority: i32,
    ) -> Result<Self, diesel::result::Error> {
        use schema::dat_files::dsl;
        diesel::insert_into(schema::dat_files::table)
            .values((
                dsl::name.eq(name),
                dsl::path.eq(path),
                dsl::core_id.eq(core.id),
                dsl::priority.eq(priority),
            ))
            .execute(conn)?;
        dsl::dat_files.order(dsl::id.desc()).first(conn)
    }

    pub fn list(conn: &mut crate::Connection) -> Result<Vec<Self>, diesel::result::Error> {
        use schema::dat_files::dsl;
        dsl::dat_files.load(conn)
    }
}
