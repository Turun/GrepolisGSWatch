use crate::{
    db::orm::{OrmGS, OrmPlayer},
    messages::{MessageFromDBToWeb, MessageFromModelToDB},
    web::CachedDBState,
};
use chrono::offset::Utc;
use std::sync::mpsc::{Receiver, Sender};

pub mod orm;

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

        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS player_disappeared (
                    date TEXT NOT NULL,
                    name TEXT NOT NULL,
                    towns INTEGER NOT NULL,
                    points INTEGER NOT NULL,
                    rank INTEGER NOT NULL,
                    alliance TEXT
                );
                CREATE TABLE IF NOT EXISTS gs_appeared (
                    date TEXT NOT NULL,
                    name TEXT NOT NULL,
                    points INTEGER NOT NULL,
                    x REAL NOT NULL,
                    y REAL NOT NULL,
                    player TEXT NOT NULL,
                    alliance TEXT NOT NULL
                );
                CREATE TABLE IF NOT EXISTS gs_disappeared (
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

        for msg in &self.rx {
            println!("Got Message to DB from Model: {msg}");
            let now = Utc::now();
            let transaction = self.conn.transaction().expect("Failed to open transaction");
            match msg {
                MessageFromModelToDB::PlayersDisappeared(players) => {
                    let mut prepared_statement = transaction
                        .prepare("INSERT INTO player_disappeared VALUES(?1, ?2, ?3, ?4, ?5, ?6)")
                        .expect("failed to prepare statement");
                    for p in players {
                        let _res = prepared_statement.execute((
                            now,
                            p.name.as_str(),
                            p.towns,
                            p.points,
                            p.rank,
                            p.alliance.as_ref().map(|a| a.1.name.as_str()),
                        ));
                    }
                }
                MessageFromModelToDB::GSAppeared(gss) => {
                    let mut prepared_statement = transaction
                        .prepare("INSERT INTO gs_appeared VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)")
                        .expect("failed to prepare statement");
                    for gs in gss {
                        let _res = prepared_statement.execute((
                            now,
                            gs.name.as_str(),
                            gs.points,
                            gs.island.0,
                            gs.island.1,
                            gs.player.as_ref().unwrap().1.name.as_str(),
                            gs.player
                                .as_ref()
                                .unwrap()
                                .1
                                .alliance
                                .as_ref()
                                .map(|a| a.1.name.as_str()),
                        ));
                    }
                }
                MessageFromModelToDB::GSDisappeared(gss) => {
                    let mut prepared_statement = transaction
                        .prepare("INSERT INTO gs_disappeared VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)")
                        .expect("failed to prepare statement");
                    for gs in gss {
                        let _res = prepared_statement.execute((
                            now,
                            gs.name.as_str(),
                            gs.points,
                            gs.island.0,
                            gs.island.1,
                            gs.player.as_ref().unwrap().1.name.as_str(),
                            gs.player
                                .as_ref()
                                .unwrap()
                                .1
                                .alliance
                                .as_ref()
                                .map(|a| a.1.name.as_str()),
                        ));
                    }
                }
            }
            transaction
                .commit()
                .expect("Failed to commit transaction for table offsets");

            // Send the new data to the web part
            // for this we turn the db table into a vector of tuples. Limited in length to keep it managable

            // TODO: make web and db more coupled and allow for filtering by position x/y

            let gs_old = self
                .conn
                .prepare("SELECT * FROM gs_disappeared ORDER BY date LIMIT 200")
                .expect("failed to prepare gs disappeared extraction statement")
                .query([])
                .expect("Failed to query db for gs appeared")
                .mapped(|r| OrmGS::try_from(r))
                .collect::<Result<Vec<_>, rusqlite::Error>>()
                .expect("Failed to collect the rows from the DB");
            let gs_new = self
                .conn
                .prepare("SELECT * FROM gs_appeared ORDER BY date LIMIT 200")
                .expect("failed to prepare gs appeared extraction statement")
                .query([])
                .expect("Failed to query db for gs appeared")
                .mapped(|r| OrmGS::try_from(r))
                .collect::<Result<Vec<_>, rusqlite::Error>>()
                .expect("Failed to collect the rows from the DB");
            let players = self
                .conn
                .prepare("SELECT * FROM players_disappeared ORDER BY date LIMIT 200")
                .expect("failed to prepare gs appeared extraction statement")
                .query([])
                .expect("Failed to query db for gs appeared")
                .mapped(|r| OrmPlayer::try_from(r))
                .collect::<Result<Vec<_>, rusqlite::Error>>()
                .expect("Failed to collect the rows from the DB");

            let res = self.tx.send(MessageFromDBToWeb::NewData(CachedDBState {
                gs_old,
                gs_new,
                players_old: players,
            }));
            if let Err(err) = res {
                eprintln!("Failed to send update to webserver: {err:?}");
            }
        }
    }
}
