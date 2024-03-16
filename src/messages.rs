//! A file to collect the messages that are sent across the channels

use core::fmt;
use std::sync::Arc;

use crate::{
    model::database::{Player, Town},
    web::CachedDBState,
};

pub enum MessageFromModelToDB {
    GSDisappeared(Vec<Arc<Town>>),
    GSAppeared(Vec<Arc<Town>>),
    PlayersDisappeared(Vec<Arc<Player>>),
}

impl fmt::Display for MessageFromModelToDB {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageFromModelToDB::GSDisappeared(list) => {
                write!(f, "GSDisappeared({})", list.len())
            }
            MessageFromModelToDB::GSAppeared(list) => {
                write!(f, "GSAppeared({})", list.len())
            }
            MessageFromModelToDB::PlayersDisappeared(list) => {
                write!(f, "PlayerDisappeared({})", list.len())
            }
        }
    }
}

pub enum MessageFromDBToWeb {
    NewData(CachedDBState),
}
