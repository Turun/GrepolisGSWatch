use crate::{
    db::orm::{OrmGS, OrmPlayer},
    messages::{MessageFromDBToWeb, MessageFromModelToDB},
    web::CachedDBState,
};
use chrono::offset::Utc;
use std::sync::mpsc::{Receiver, Sender};

pub mod orm;

use tracing::error;
use tracing::{info, trace};

pub struct DB {
    rx: Receiver<MessageFromModelToDB>,
    tx: Sender<MessageFromDBToWeb>,
    conn: rusqlite::Connection,
}

impl DB {
    pub fn new(rx: Receiver<MessageFromModelToDB>, tx: Sender<MessageFromDBToWeb>) -> Self {
        let conn =
            rusqlite::Connection::open("db.sqlite").expect("failed to open the database file");

        Self { rx, tx, conn }
    }

    pub fn start(&mut self) {
        //ensure DB Schema

        self.ensure_db_schema();

        // bring the webserver up to speed on the data we already have.
        self.send_update_to_webserver();

        for msg in &self.rx {
            let now = Utc::now();
            let transaction = self.conn.transaction().expect("Failed to open transaction");
            match &msg {
                MessageFromModelToDB::PlayersDisappeared(players) => {
                    info!("Got Message from Model to DB: {msg}");
                    if !players.is_empty() {
                        let mut prepared_statement = transaction
                            .prepare(
                                "INSERT INTO player_disappeared VALUES(?1, ?2, ?3, ?4, ?5, ?6)",
                            )
                            .expect("failed to prepare statement");
                        for p in players {
                            trace!("Inserting {p:?} into DB");
                            let res = prepared_statement.execute((
                                now,
                                p.name.as_str(),
                                p.towns,
                                p.points,
                                p.rank,
                                p.alliance.as_deref(),
                            ));
                            if let Err(err) = res {
                                error!("Failed to insert player into DB: {err:?}");
                            }
                        }
                    }
                }
                MessageFromModelToDB::GSAppeared(gss) => {
                    info!("Got Message from Model to DB: {msg}");
                    if !gss.is_empty() {
                        let mut prepared_statement = transaction
                            .prepare("INSERT INTO gs_appeared VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)")
                            .expect("failed to prepare statement");
                        for gs in gss {
                            trace!("Inserting {gs:?} into DB.gs_appeared");
                            let res = prepared_statement.execute((
                                now,
                                gs.name.as_str(),
                                gs.points,
                                gs.x,
                                gs.y,
                                gs.player_name.as_deref(),
                                gs.alliance_name.as_deref(),
                            ));
                            if let Err(err) = res {
                                error!("Failed to insert gs into DB: {err:?}");
                            }
                        }
                    }
                }
                MessageFromModelToDB::GSConquered(gss) => {
                    info!("Got Message from Model to DB: {msg}");
                    if !gss.is_empty() {
                        let mut prepared_statement = transaction
                            .prepare("INSERT INTO gs_conquered VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)")
                            .expect("failed to prepare statement");
                        for gs in gss {
                            trace!("Inserting {gs:?} into DB.gs_conquered");
                            let res = prepared_statement.execute((
                                now,
                                gs.name.as_str(),
                                gs.points,
                                gs.x,
                                gs.y,
                                gs.player_name.as_deref(),
                                gs.alliance_name.as_deref(),
                            ));
                            if let Err(err) = res {
                                error!("Failed to insert gs into DB: {err:?}");
                            }
                        }
                    }
                }
            }
            transaction
                .commit()
                .expect("Failed to commit transaction for table offsets");

            // Send the new data to the web part
            // for this we turn the db table into a vector of tuples. Limited in length to keep it managable

            // TODO: make web and db more coupled and allow for filtering by position x/y
            self.send_update_to_webserver();
        }
    }

    fn ensure_db_schema(&self) {
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS player_disappeared (
                    date TEXT NOT NULL,
                    name TEXT NOT NULL,
                    towns INTEGER NOT NULL,
                    points INTEGER NOT NULL,
                    rank INTEGER NOT NULL,
                    alliance TEXT
                );",
                (),
            )
            .expect("Failed to define the Database Schema");
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS gs_appeared (
                    date TEXT NOT NULL,
                    name TEXT NOT NULL,
                    points INTEGER NOT NULL,
                    x REAL NOT NULL,
                    y REAL NOT NULL,
                    player TEXT NOT NULL,
                    alliance TEXT
                );",
                (),
            )
            .expect("Failed to define the Database Schema");
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS gs_conquered(
                    date TEXT NOT NULL,
                    name TEXT NOT NULL,
                    points INTEGER NOT NULL,
                    x REAL NOT NULL,
                    y REAL NOT NULL,
                    player TEXT NOT NULL,
                    alliance TEXT 
                );",
                (),
            )
            .expect("Failed to define the Database Schema");
    }

    fn send_update_to_webserver(&self) {
        let gs_conquered = self
            .conn
            .prepare("SELECT * FROM gs_conquered ORDER BY date LIMIT 200")
            .expect("failed to prepare gs conquered extraction statement")
            .query([])
            .expect("Failed to query db for gs appeared")
            .mapped(|r| OrmGS::try_from(r))
            .collect::<Result<Vec<_>, rusqlite::Error>>()
            .expect("Failed to collect the rows from the DB");
        let gs_appeared = self
            .conn
            .prepare("SELECT * FROM gs_appeared ORDER BY date LIMIT 200")
            .expect("failed to prepare gs appeared extraction statement")
            .query([])
            .expect("Failed to query db for gs appeared")
            .mapped(|r| OrmGS::try_from(r))
            .collect::<Result<Vec<_>, rusqlite::Error>>()
            .expect("Failed to collect the rows from the DB");
        let players_left = self
            .conn
            .prepare("SELECT * FROM player_disappeared ORDER BY date LIMIT 200")
            .expect("failed to prepare gs appeared extraction statement")
            .query([])
            .expect("Failed to query db for gs appeared")
            .mapped(|r| OrmPlayer::try_from(r))
            .collect::<Result<Vec<_>, rusqlite::Error>>()
            .expect("Failed to collect the rows from the DB");

        let res = self.tx.send(MessageFromDBToWeb::NewData(CachedDBState {
            gs_conquered,
            gs_appeared,
            players_left,
        }));
        if let Err(err) = res {
            error!("Failed to send update to webserver: {err:?}");
        }
    }
}
