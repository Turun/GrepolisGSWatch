use std::collections::HashMap;

use chrono::{DateTime, Utc};
use rusqlite::Row;

use crate::model::database::{Alliance, Player, Town};

#[allow(clippy::module_name_repetitions)]
pub struct OrmGS {
    pub date: DateTime<Utc>,
    pub name: String,
    pub points: u16,
    pub x: f32,
    pub y: f32,
    pub player_name: Option<String>,
    pub alliance_name: Option<String>,
}

impl
    From<(
        DateTime<Utc>,
        &Town,
        &HashMap<u32, Player>,
        &HashMap<u32, Alliance>,
    )> for OrmGS
{
    fn from(
        (now, town, players, alliances): (
            DateTime<Utc>,
            &Town,
            &HashMap<u32, Player>,
            &HashMap<u32, Alliance>,
        ),
    ) -> Self {
        let opt_player = town.player_id.and_then(|id| players.get(&id));
        Self {
            date: now,
            name: town.name.clone(),
            points: town.points,
            x: town.actual_x,
            y: town.actual_y,
            player_name: opt_player.map(|p| p.name.clone()),
            alliance_name: opt_player
                .and_then(|p| p.alliance_id)
                .and_then(|id| alliances.get(&id))
                .map(|a| a.name.clone()),
        }
    }
}

impl<'a> TryFrom<&Row<'a>> for OrmGS {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'a>) -> Result<Self, Self::Error> {
        Ok(Self {
            date: row.get(0).unwrap(),
            name: row.get(1).unwrap(),
            points: row.get(2).unwrap(),
            x: row.get(3).unwrap(),
            y: row.get(4).unwrap(),
            player_name: row.get(5).unwrap(),
            alliance_name: row.get(6).unwrap(),
        })
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct OrmPlayer {
    pub date: DateTime<Utc>,
    pub name: String,
    pub towns: u16,
    pub points: u32,
    pub rank: u16,
    pub alliance: Option<String>,
}

impl From<(DateTime<Utc>, &Player, &HashMap<u32, Alliance>)> for OrmPlayer {
    fn from((now, player, alliances): (DateTime<Utc>, &Player, &HashMap<u32, Alliance>)) -> Self {
        Self {
            date: now,
            name: player.name.clone(),
            towns: player.towns,
            points: player.points,
            rank: player.rank,
            alliance: player
                .alliance_id
                .and_then(|id| alliances.get(&id))
                .map(|a| a.name.clone()),
        }
    }
}

impl<'a> TryFrom<&Row<'a>> for OrmPlayer {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'a>) -> Result<Self, Self::Error> {
        Ok(Self {
            date: row.get(0).unwrap(),
            name: row.get(1).unwrap(),
            towns: row.get(2).unwrap(),
            points: row.get(3).unwrap(),
            rank: row.get(4).unwrap(),
            alliance: row.get(5).unwrap(),
        })
    }
}
