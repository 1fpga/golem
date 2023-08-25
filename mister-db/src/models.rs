use diesel::prelude::*;

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::schema::cores)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Core {
    pub id: i32,

    /// The name of this core.
    pub name: String,

    /// Overwritten name by the user.
    pub user_name: Option<String>,

    /// The path to the core's image.
    pub path: String,

    /// A list of comma-separated authors of the form "Author Name <email@address>".
    pub author: String,

    /// A home URL.
    pub home: String,

    /// A description of the core.
    pub description: String,

    /// A comma-separated list of file extensions that this core supports.
    pub extensions: String,

    /// When this core was added to the database.
    pub created_at: chrono::NaiveDateTime,

    /// The last time this core was updated.
    pub updated_at: chrono::NaiveDateTime,
}
