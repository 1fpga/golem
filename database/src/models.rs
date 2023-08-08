use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[table_name = crate::schema::cores]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Core {
    pub id: i32,
    pub name: String,
    pub path: String,
    pub description: String,
    pub extensions: Vec<String>,
}
