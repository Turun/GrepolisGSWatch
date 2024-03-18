#![warn(clippy::pedantic)]
#![allow(clippy::needless_return)]

use std::{panic, process, sync::mpsc, thread};

use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::filter::LevelFilter;

use crate::{
    db::DB,
    messages::{MessageFromDBToWeb, MessageFromModelToDB},
    model::Model,
    web::Web,
};

mod db;
mod messages;
mod model;
mod web;

fn main() {
    // setup logging
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::WARN.into())
        .from_env()
        .expect("failed to create logging filter")
        .add_directive("grepolis_diff_server=trace".parse().unwrap());
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .compact()
        .init();

    // all threads communicate via message passing
    let (tx_model_to_db, rx_db_from_model) = mpsc::channel::<MessageFromModelToDB>();
    let (tx_db_to_web, rx_web_from_db) = mpsc::channel::<MessageFromDBToWeb>();

    // from https://stackoverflow.com/a/36031130/14053391
    // allow the db to crash the whole application if it encounters any issue
    let orig_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        // invoke the default handler and exit the process
        orig_hook(panic_info);
        process::exit(1);
    }));

    // thread 1:
    // fetches a new state regularly and writes to the database what changed
    // If new data has been detected (i.e. any changes) then
    //  - the diff is computed,
    //  - any changes are sent to the DB Thread
    //  - optional: the new state is saved to allow a comparion immediately after reboot
    let handle_model = thread::spawn(move || {
        Model::new(tx_model_to_db).start();
    });

    // thread 2:
    // the Database of changes. In a nice format so that the websever can present the data
    // it responds to requests from the webserver
    // and accepts updates from the backend.
    // persisted on disk
    let handle_db = thread::spawn(move || {
        DB::new(rx_db_from_model, tx_db_to_web).start();
    });

    // thread 3:
    // the webserver, handles request and reports back the data from the database.
    // keeps all required data locally. This data is updated by the DB, whenever the DB receives new data from the backend
    let handle_web = thread::spawn(move || {
        Web::new(rx_web_from_db).start();
    });

    let _res = handle_web.join();
    let _res = handle_db.join();
    let _res = handle_model.join();
}
