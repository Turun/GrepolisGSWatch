use std::{ops::Deref, sync::Arc};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Offset {
    pub typ: u8,
    pub x: u16,
    pub y: u16,
    pub slot_number: u8,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Island {
    pub id: u32,
    pub x: u16,
    pub y: u16,
    pub typ: u8,
    pub towns: u8,
    pub ressource_plus: String,
    pub ressource_minus: String,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Alliance {
    pub id: u32,
    pub name: String,
    pub points: u32,
    pub towns: u32,
    pub members: u16,
    pub rank: u16,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Player {
    pub id: u32,
    pub name: String,
    pub alliance: Option<(u32, Arc<Alliance>)>, // link player.alliance_id == alliance.id
    pub points: u32,
    pub rank: u16,
    pub towns: u16,
}

#[derive(Clone, PartialEq)]
pub struct Town {
    pub id: u32,
    pub name: String,
    pub points: u16,
    pub player: Option<(u32, Arc<Player>)>, // link town.player_id == player.id
    pub island: (u16, u16, Arc<Island>),    // link town.x = island.x && town.y == island.y
    pub offset: (u8, Arc<Offset>), // link town.slot_number = offset.slot_number && offset.type == island.type
    pub actual_x: f32,
    pub actual_y: f32, // computed from the linked island and offset
}

impl Eq for Town {}
impl std::hash::Hash for Town {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.name.hash(state);
        self.points.hash(state);
        self.player.hash(state);
        self.island.hash(state);
        self.offset.hash(state);
        self.actual_x.to_bits().hash(state);
        self.actual_y.to_bits().hash(state);
    }
}

#[derive(PartialEq)]
pub struct DataTable {
    pub offsets: Vec<Arc<Offset>>,
    pub islands: Vec<Arc<Island>>,
    pub alliances: Vec<Arc<Alliance>>,
    pub players: Vec<Arc<Player>>,
    pub towns: Vec<Arc<Town>>,
}

impl DataTable {
    pub fn get_ghost_towns(&self) -> Vec<Arc<Town>> {
        self.towns
            .iter()
            .filter(|&t| t.deref().player.is_none())
            .cloned()
            .collect()
    }
}
