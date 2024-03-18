use chrono::Utc;
use std::{collections::HashSet, sync::mpsc::Sender, thread, time};
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
            let res = DataTable::create_for_world("de99");
            match res {
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
            let now = Utc::now();
            let mut tracked_any_updates = false;

            let gs_ids_new = state_new.get_ghost_town_ids();
            let gs_ids_old = state_old.get_ghost_town_ids();

            let diff_appeared = gs_ids_new.difference(&gs_ids_old); // basically new - old
            let diff_conquered = gs_ids_old.difference(&gs_ids_new); // basically old - new

            let gs_appeared: Vec<_> = diff_appeared
                .filter_map(|id| state_old.towns.get(id))
                .map(|town| OrmGS::from((now, town, &state_old.players, &state_old.alliances)))
                .collect();
            if !gs_appeared.is_empty() {
                tracked_any_updates = true;
                let res = self.tx.send(MessageFromModelToDB::GSAppeared(gs_appeared));
                if let Err(err) = res {
                    error!("Failed to send list of appeared GS to Database: {}", err);
                }
            }

            let gs_conquered: Vec<_> = diff_conquered
                .filter_map(|id| state_new.towns.get(id))
                .map(|town| OrmGS::from((now, town, &state_new.players, &state_new.alliances)))
                .collect();
            if !gs_conquered.is_empty() {
                tracked_any_updates = true;
                let res = self
                    .tx
                    .send(MessageFromModelToDB::GSConquered(gs_conquered));
                if let Err(err) = res {
                    error!("Failed to send list of conquered GS to Database: {}", err);
                }
            }

            // Determine which player no longer exists
            let player_ids_old: HashSet<_> = state_old.players.keys().collect();
            let player_ids_new: HashSet<_> = state_new.players.keys().collect();

            let players_disappeared: Vec<_> = player_ids_old
                .difference(&player_ids_new)
                .filter_map(|id| state_old.players.get(id))
                .map(|player| OrmPlayer::from((now, player, &state_old.alliances)))
                .collect();
            if !players_disappeared.is_empty() {
                tracked_any_updates = true;
                let res = self.tx.send(MessageFromModelToDB::PlayersDisappeared(
                    players_disappeared,
                ));
                if let Err(err) = res {
                    error!(
                        "Failed to send list of disappeared Players to Database: {}",
                        err
                    );
                }
            }

            if !tracked_any_updates {
                info!("No changes this time");
            }

            state_old = state_new;
        }
    }
}
