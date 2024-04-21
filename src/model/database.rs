use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Offset {
    pub typ: u8,
    pub x: u16,
    pub y: u16,
    pub slot_number: u8,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Island {
    pub id: u32,
    pub x: u16,
    pub y: u16,
    pub typ: u8,
    pub towns: u8,
    pub ressource_plus: String,
    pub ressource_minus: String,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Alliance {
    pub id: u32,
    pub name: String,
    pub points: u32,
    pub towns: u32,
    pub members: u16,
    pub rank: u16,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Player {
    pub id: u32,
    pub name: String,
    pub alliance_id: Option<u32>, // link player.alliance_id == alliance.id
    pub points: u32,
    pub rank: u16,
    pub towns: u16,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Town {
    pub id: u32,
    pub name: String,
    pub points: u16,
    pub player_id: Option<u32>, // link town.player_id == player.id
    pub island_xy: (u16, u16),  // link town.x = island.x && town.y == island.y
    pub offset_slotnumber: u8, // link town.slot_number = offset.slot_number && offset.type == island.type
    pub actual_x: f32,
    pub actual_y: f32, // computed from the linked island and offset
}

impl Eq for Town {}
impl std::hash::Hash for Town {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.name.hash(state);
        self.points.hash(state);
        self.player_id.hash(state);
        self.island_xy.hash(state);
        self.offset_slotnumber.hash(state);
        self.actual_x.to_bits().hash(state);
        self.actual_y.to_bits().hash(state);
    }
}

#[derive(PartialEq, Serialize, Deserialize)]
pub struct DataTable {
    pub loaded: DateTime<Utc>,
    pub offsets: HashMap<u8, Offset>,
    pub islands: HashMap<(u16, u16), Island>,
    pub alliances: HashMap<u32, Alliance>,
    pub players: HashMap<u32, Player>,
    pub towns: HashMap<u32, Town>,
}

impl DataTable {
    pub fn get_ghost_town_ids(&self) -> HashSet<u32> {
        self.towns
            .values()
            .filter(|t| t.player_id.is_none())
            .map(|t| t.id)
            .collect()
    }
}
