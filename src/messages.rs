//! A file to collect the messages that are sent across the channels

use core::fmt;
use std::sync::Arc;

use crate::{
    db::orm::{OrmGS, OrmPlayer},
    model::database::{Player, Town},
    web::CachedDBState,
};

pub enum MessageFromModelToDB {
    GSDisappeared(Vec<OrmGS>),
    GSAppeared(Vec<OrmGS>),
    PlayersDisappeared(Vec<OrmPlayer>),
}

impl fmt::Display for MessageFromModelToDB {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageFromModelToDB::GSDisappeared(list) => {
                write!(f, "GSDisappeared(len={})", list.len())
            }
            MessageFromModelToDB::GSAppeared(list) => {
                write!(f, "GSAppeared(len={})", list.len())
            }
            MessageFromModelToDB::PlayersDisappeared(list) => {
                write!(f, "PlayerDisappeared(len={})", list.len())
            }
        }
    }
}

pub enum MessageFromDBToWeb {
    NewData(CachedDBState),
}
impl fmt::Display for MessageFromDBToWeb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageFromDBToWeb::NewData(data) => {
                write!(
                    f,
                    "NewData(len_gs_appeared={}, len_gs_conquered={}, len_players_left={})",
                    data.gs_appeared.len(),
                    data.gs_conquered.len(),
                    data.players_left.len()
                )
            }
        }
    }
}
