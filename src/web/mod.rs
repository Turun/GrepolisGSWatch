use std::sync::{mpsc::Receiver, Arc, Mutex};

use axum::{extract::State, routing::get, Router};
use tracing::debug;
use tracing::info;

use crate::{
    db::orm::{OrmGS, OrmPlayer},
    messages::MessageFromDBToWeb,
};

pub struct CachedDBState {
    pub gs_conquered: Vec<OrmGS>,
    pub gs_appeared: Vec<OrmGS>,
    pub players_left: Vec<OrmPlayer>,
}

pub struct Web {
    rx: Receiver<MessageFromDBToWeb>,
    cached_db_state: Arc<Mutex<CachedDBState>>,
}

impl Web {
    pub fn new(rx: Receiver<MessageFromDBToWeb>) -> Self {
        Self {
            rx,
            cached_db_state: Arc::new(Mutex::new(CachedDBState {
                gs_conquered: Vec::new(),
                gs_appeared: Vec::new(),
                players_left: Vec::new(),
            })),
        }
    }

    pub fn start(self) {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

        let cache_server = Arc::clone(&self.cached_db_state);
        rt.spawn(async {
            info!("Starting server to listen on [::]:10204");
            // setup and start the axum server
            let app = Router::new()
                .route("/", get(Self::serve_main_page))
                .with_state(cache_server);
            axum::Server::bind(&"[::]:10204".parse().unwrap())
                .serve(app.into_make_service())
                .await
                .unwrap();
        });

        for msg in self.rx {
            info!("Got Message from DB to Web: {}", msg);
            match msg {
                MessageFromDBToWeb::NewData(state) => {
                    *self.cached_db_state.lock().unwrap() = state;
                }
            }
        }
    }

    async fn serve_main_page(
        State(cache): State<Arc<Mutex<CachedDBState>>>,
    ) -> Result<String, axum::http::StatusCode> {
        debug!("Serving a request!");
        let inner = cache.lock().unwrap();
        Ok(format!(
            "Hello World! DB Contents: OldGS {} / NewGS {} / Players {}",
            inner.gs_conquered.len(),
            inner.gs_appeared.len(),
            inner.players_left.len()
        ))
    }
}
