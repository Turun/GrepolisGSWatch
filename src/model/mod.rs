use chrono::Utc;
use std::{sync::mpsc::Sender, thread, time};
use tracing::{error, info, warn};

use crate::{
    db::orm::{OrmGS, OrmPlayer},
    messages::MessageFromModelToDB,
};

use self::database::DataTable;

pub mod database;
mod download;
mod offset_data;

pub struct Model {
    tx: Sender<MessageFromModelToDB>,
}

impl Model {
    pub fn new(tx: Sender<MessageFromModelToDB>) -> Self {
        Self { tx }
    }

    fn get_datatable_for_sure() -> DataTable {
        loop {
            // TODO do this for more servers
            let dt = DataTable::create_for_world("de99");
            match dt {
                Ok(dt) => {
                    info!("Successfully loaded a new DataTable");
                    break dt;
                }
                Err(err) => {
                    warn!("Failed to load DB: {:?}", err);
                    thread::sleep(time::Duration::from_secs(60));
                }
            }
        }
    }

    pub fn start(self) {
        let mut state_old = Self::get_datatable_for_sure();
        loop {
            thread::sleep(time::Duration::from_secs(5 * 60));

            let state_new = Self::get_datatable_for_sure();
            if state_new == state_old {
                continue;
            }

            let gs_new = state_new.get_ghost_towns();
            let gs_old = state_old.get_ghost_towns();

            // Determine which GS are new
            let mut gs_appeared = Vec::new();
            for gsn in &gs_new {
                let has_a_match = gs_old.iter().any(|gs| gs.id == gsn.id);
                if !has_a_match {
                    gs_appeared.push(OrmGS::from((now, *gsn)));
                }
            }
            let res = self.tx.send(MessageFromModelToDB::GSAppeared(gs_appeared));
            if let Err(err) = res {
                error!("Failed to send list of appeared GS to Database: {}", err);
            }

            // Determine which GS are no longer present
            let mut gs_disappeared = Vec::new();
            for gso in &gs_old {
                let has_match = gs_new.iter().any(|gs| gs.id == gso.id);
                if !has_match {
                    gs_disappeared.push(OrmGS::from((now, *gso)));
                }
            }
            let res = self
                .tx
                .send(MessageFromModelToDB::GSDisappeared(gs_disappeared));
            if let Err(err) = res {
                error!("Failed to send list of disappeared GS to Database: {}", err);
            }

            // Determine which player no longer exists
            let mut players_disappeared = Vec::new();
            for (ido, po) in &state_old.players {
                let has_match = state_new.players.iter().any(|(idn, _)| ido == idn);
                if !has_match {
                    players_disappeared.push(OrmPlayer::from((now, po)));
                }
            }
            let res = self.tx.send(MessageFromModelToDB::PlayersDisappeared(
                players_disappeared,
            ));
            if let Err(err) = res {
                error!(
                    "Failed to send list of disappeared Players to Database: {}",
                    err
                );
            }

            state_old = state_new;
        }
    }
}
