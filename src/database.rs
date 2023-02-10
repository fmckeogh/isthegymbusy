//! Reusable database functionality
//!
//! Most queries are still done in route implementations but deduplicated in this module.

use {
    crate::{
        entity::games::{self, Entity as Game},
        error::Error,
        game_code::GameCode,
        util::utc_now,
    },
    sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter},
};

/// Finds the game ID given a game code
///
/// Game must have not started yet
pub async fn lookup_prestart_game_code(
    database: &DatabaseConnection,
    game_code: &GameCode,
) -> Result<i32, Error> {
    if let Some(model) = Game::find()
        .filter(games::Column::Code.eq(game_code.as_str()))
        .filter(games::Column::StartTime.gt(utc_now()))
        .one(database)
        .await?
    {
        return Ok(model.id);
    } else {
        return Err(Error::ReadyGameNotFound);
    };
}
