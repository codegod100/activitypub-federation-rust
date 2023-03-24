use crate::{
    database::Database,
    http::{http_get_user, http_post_user_inbox, webfinger},
    objects::{person::DbUser, post::DbPost},
    utils::generate_object_id,
};
use activitypub_federation::config::{FederationConfig, FederationMiddleware};
use axum::{
    extract::Json,
    routing::{get, post, put},
    Router,
};
use error::Error;
use serde::Deserialize;
use std::{
    net::ToSocketAddrs,
    sync::{Arc, Mutex},
};
use tracing::log::{info, LevelFilter};

mod activities;
mod database;
mod error;
#[allow(clippy::diverging_sub_expression, clippy::items_after_statements)]
mod http;
mod objects;
mod utils;

const DOMAIN: &str = "social.vera.pink";
const LOCAL_USER_NAME: &str = "v";
const BIND_ADDRESS: &str = "127.0.0.1:8003";

use axum_macros::debug_handler;

#[derive(Deserialize, Debug)]
struct Content {
    body: String,
    msgtype: String,
}

#[derive(Deserialize, Debug)]
struct Event {
    content: Content,
    event_id: String,
    origin_server_ts: i64,
    room_id: String,
    sender: String,
    #[serde(rename = "type")]
    kind: String,
}

#[derive(Deserialize, Debug)]
struct Transaction {
    events: Vec<Event>,
}

#[actix_rt::main]
async fn main() -> Result<(), Error> {
    env_logger::builder()
        .filter_level(LevelFilter::Warn)
        .filter_module("activitypub_federation", LevelFilter::Debug)
        .filter_module("live_federation", LevelFilter::Debug)
        .format_timestamp(None)
        .init();

    info!("Setup local user and database");
    let local_user = DbUser::new(DOMAIN, LOCAL_USER_NAME)?;
    let database = Arc::new(Database {
        users: Mutex::new(vec![local_user]),
    });

    info!("Setup configuration");
    let config = FederationConfig::builder()
        .domain(DOMAIN)
        .app_data(database)
        .build()?;

    info!("Listen with HTTP server on {BIND_ADDRESS}");
    let config = config.clone();
    let app = Router::new()
        .route("/:user", get(http_get_user))
        .route("/:user/inbox", post(http_post_user_inbox))
        .route("/.well-known/webfinger", get(webfinger))
        .route(
            "/_matrix/app/v1/transactions/:transaction",
            put(handle_matrix_tx),
        )
        .layer(FederationMiddleware::new(config));

    let addr = BIND_ADDRESS
        .to_socket_addrs()?
        .next()
        .expect("Failed to lookup domain name");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

#[debug_handler]
async fn handle_matrix_tx(body: String) -> String {
    println!("in matrix handler");
    let tx: Transaction = serde_json::from_str(&body).unwrap();
    // let out = format!("Transaction: {:#?}", tx);
    println!("{:#?}\n\n{:#?}", body, tx);
    body
}
