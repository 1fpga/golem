use crate::models::Core;
use crate::schema;
use diesel::prelude::*;
use std::path::Path;
use strum::{Display, EnumCount, FromRepr};

#[derive(Copy, Clone, Default, Debug, FromRepr, Display, EnumCount)]
pub enum GameOrder {
    NameAsc,
    NameDesc,
    CoreAsc,
    LastAdded,
    #[default]
    LastPlayed,
    Favorite,
}

impl GameOrder {
    pub fn next(self) -> Self {
        Self::from_repr((self as usize + 1) % Self::COUNT).unwrap()
    }

    pub fn previous(self) -> Self {
        Self::from_repr((self as usize).checked_sub(1).unwrap_or(Self::COUNT - 1)).unwrap()
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NameAsc => "Name (A-Z)",
            Self::NameDesc => "Name (Z-A)",
            Self::CoreAsc => "Core (A-Z)",
            Self::LastAdded => "Last Added",
            Self::LastPlayed => "Last Played",
            Self::Favorite => "Favorites",
        }
    }
}

#[derive(Clone, Debug, Queryable, Selectable, Identifiable)]
#[diesel(table_name = schema::games)]
#[diesel(belongs_to(Core))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Game {
    pub id: i32,

    /// The name of this game.
    pub name: String,

    /// The core that can load this game. If unset, there's no core downloaded that can
    /// read it, the user has to download a core.
    pub core_id: Option<i32>,

    /// The path to the game's rom. This is None if the core does not load ROMs
    /// (e.g. Arcade), in which case there should be only one row per core.
    pub path: Option<String>,

    /// A description of the game.
    pub description: String,

    /// The last time this core was played.
    pub last_played: Option<chrono::NaiveDateTime>,

    /// The time this was added to the Library.
    pub added_at: chrono::NaiveDateTime,

    /// Whether this core is a favorite.
    pub favorite: bool,
}

impl Game {
    pub fn get(
        conn: &mut crate::Connection,
        id: i32,
    ) -> Result<Option<Self>, diesel::result::Error> {
        schema::games::table.find(id).first(conn).optional()
    }

    pub fn get_by_name(
        conn: &mut crate::Connection,
        name: &str,
    ) -> Result<Option<Self>, diesel::result::Error> {
        use schema::games::dsl;
        schema::games::table
            .select(schema::games::all_columns)
            .filter(dsl::name.eq(name))
            .first(conn)
            .optional()
    }

    pub fn count(conn: &mut crate::Connection) -> Result<i64, diesel::result::Error> {
        use schema::games::dsl::*;
        games.count().get_result(conn)
    }

    pub fn create(
        conn: &mut crate::Connection,
        name: String,
        core: &Core,
        path: impl AsRef<Path>,
        description: String,
    ) -> Result<Self, diesel::result::Error> {
        use schema::games::dsl;

        diesel::insert_into(schema::games::table)
            .values((
                dsl::name.eq(&name),
                dsl::core_id.eq(core.id),
                dsl::path.eq(path.as_ref().to_str().unwrap()),
                dsl::description.eq(&description),
                dsl::added_at.eq(chrono::Utc::now().naive_utc()),
            ))
            .execute(conn)?;

        dsl::games.order(dsl::id.desc()).first(conn)
    }

    pub fn play(&mut self, conn: &mut crate::Connection) -> Result<(), diesel::result::Error> {
        use schema::games::dsl;

        let datetime = chrono::Utc::now().naive_utc();
        diesel::update(dsl::games.find(self.id))
            .set(dsl::last_played.eq(&datetime))
            .execute(conn)?;

        self.last_played = Some(datetime);

        Ok(())
    }

    pub fn favorite(&mut self, conn: &mut crate::Connection) -> Result<(), diesel::result::Error> {
        use schema::games::dsl;

        diesel::update(dsl::games.find(self.id))
            .set(dsl::favorite.eq(!self.favorite))
            .execute(conn)?;
        self.favorite = !self.favorite;

        Ok(())
    }

    pub fn delete(&mut self, conn: &mut crate::Connection) -> Result<(), diesel::result::Error> {
        diesel::delete(schema::games::table.find(self.id)).execute(conn)?;
        Ok(())
    }

    pub fn list(
        conn: &mut crate::Connection,
        page: i64,
        limit: i64,
        order_by: GameOrder,
    ) -> Result<Vec<Self>, diesel::result::Error> {
        use schema::games::dsl;
        let mut query = dsl::games
            .inner_join(schema::cores::table)
            .offset(page * limit)
            .limit(limit)
            .into_boxed();

        query = match order_by {
            GameOrder::NameAsc => query.order(dsl::name.asc()),
            GameOrder::NameDesc => query.order(dsl::name.desc()),
            GameOrder::CoreAsc => query.order(schema::cores::dsl::name.asc()),
            GameOrder::LastAdded => query.order(dsl::added_at.desc()),
            GameOrder::LastPlayed => query.order(dsl::last_played.desc()),
            GameOrder::Favorite => query
                .filter(dsl::favorite.eq(true))
                .order(dsl::last_played.desc()),
        };

        query.select(schema::games::all_columns).load::<Self>(conn)
    }
}
