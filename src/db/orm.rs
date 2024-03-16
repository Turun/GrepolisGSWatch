use chrono::{DateTime, Utc};
use rusqlite::Row;

pub struct OrmGS {
    date: DateTime<Utc>,
    name: String,
    points: u16,
    x: f32,
    y: f32,
    player_name: String,
    alliance_name: Option<String>,
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

pub struct OrmPlayer {
    date: DateTime<Utc>,
    name: String,
    towns: u16,
    points: u32,
    rank: u16,
    alliance: Option<String>,
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
