use std::{collections::HashMap, ops::Deref, sync::Arc};

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
pub struct Player<'a> {
    pub id: u32,
    pub name: String,
    pub alliance: Option<(u32, &'a Alliance)>, // link player.alliance_id == alliance.id
    pub points: u32,
    pub rank: u16,
    pub towns: u16,
}

#[derive(Clone, PartialEq)]
pub struct Town<'a> {
    pub id: u32,
    pub name: String,
    pub points: u16,
    pub player: Option<(u32, &'a Player<'a>)>, // link town.player_id == player.id
    pub island: (u16, u16, &'a Island),        // link town.x = island.x && town.y == island.y
    pub offset: (u8, &'a Offset), // link town.slot_number = offset.slot_number && offset.type == island.type
    pub actual_x: f32,
    pub actual_y: f32, // computed from the linked island and offset
}

impl<'a> Eq for Town<'a> {}
impl<'a> std::hash::Hash for Town<'a> {
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
pub struct DataTable<'a> {
    pub offsets: HashMap<u8, Offset>,
    pub islands: HashMap<(u16, u16), Island>,
    pub alliances: HashMap<u32, Alliance>,
    pub players: HashMap<u32, Player<'a>>,
    pub towns: HashMap<u32, Town<'a>>,
}

impl<'a> DataTable<'a> {
    /// create a new `DataTable` with no content
    pub fn empty() -> Self {
        Self {
            offsets: HashMap::new(),
            islands: HashMap::new(),
            alliances: HashMap::new(),
            players: HashMap::new(),
            towns: HashMap::new(),
        }
    }

    pub fn get_ghost_towns(&'a self) -> Vec<&'a Town<'a>> {
        self.towns
            .iter()
            .map(|(_, t)| t)
            .filter(|t| t.player.is_none())
            .collect()
    }
}
