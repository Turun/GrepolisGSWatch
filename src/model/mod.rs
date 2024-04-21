use anyhow::Context;
use chrono::Utc;
use std::{collections::HashSet, sync::mpsc::Sender, thread, time};
use tracing::{error, info, warn};

use postcard;

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

    /// load the file `old_state.bin` from disk into a `DataTable`
    fn load_state() -> anyhow::Result<DataTable> {
        let bytes = std::fs::read("./state_old.bin")
            .with_context(|| "Failed to read the old state from disk!")?;
        let dt = postcard::from_bytes(&bytes).with_context(|| "Failed to parse the old state!")?;
        return Ok(dt);
    }

    /// save the given `DataTable` to the file `old_state.bin` on disk
    fn save_state(dt: &DataTable) -> anyhow::Result<()> {
        let bytes = postcard::to_allocvec(dt)
            .with_context(|| "failed to convert the Datatable to postcard format")?;
        std::fs::write("./state_old.bin", bytes)
            .with_context(|| "Failed to write the old state to disk")?;
        return Ok(());
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
        let mut state_old = Self::load_state().unwrap_or_else(|err| {
            error!("{:?}", err);
            Self::get_datatable_for_sure()
        });
        loop {
            // ensure we do not compare datatables that were fetched less than one hour apart from each other.
            let now = Utc::now();
            let delta = now - state_old.loaded;
            let min_sleep = chrono::Duration::try_hours(1).unwrap();
            thread::sleep(
                delta
                    .min(min_sleep)
                    .to_std()
                    .unwrap_or(time::Duration::from_secs(1 * 60 * 60)),
            );

            let state_new = Self::get_datatable_for_sure();
            let mut tracked_any_updates = false;

            let gs_ids_new = state_new.get_ghost_town_ids();
            let gs_ids_old = state_old.get_ghost_town_ids();

            let diff_appeared = gs_ids_new.difference(&gs_ids_old); // basically new - old
            let diff_conquered = gs_ids_old.difference(&gs_ids_new); // basically old - new

            let gs_appeared: Vec<_> = diff_appeared
                .filter_map(|id| state_old.towns.get(id))
                .map(|town| {
                    OrmGS::from((
                        state_new.loaded,
                        town,
                        &state_old.players,
                        &state_old.alliances,
                    ))
                })
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
                .map(|town| {
                    OrmGS::from((
                        state_new.loaded,
                        town,
                        &state_new.players,
                        &state_new.alliances,
                    ))
                })
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
                .map(|player| OrmPlayer::from((state_new.loaded, player, &state_old.alliances)))
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
            let res = Self::save_state(&state_old);
            if let Err(err) = res {
                error!("{:?}", err);
            }
        }
    }
}
