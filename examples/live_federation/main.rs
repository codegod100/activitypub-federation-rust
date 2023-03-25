use crate::{
    activities::create_post::CreatePost,
    database::{Database, DatabaseHandle},
    http::{http_get_user, http_post_user_inbox, webfinger},
    objects::{
        person::DbUser,
        post::{DbPost, Mention, Note},
    },
    utils::generate_object_id,
};
use activitypub_federation::config::{Data, FederationConfig, FederationMiddleware};
use activitystreams_kinds::public;
use axum::{
    extract::Json,
    response::Html,
    routing::{get, post, put},
    Router,
};
use error::Error;
use regex::Regex;
use reqwest::Url;
use rocksdb::DB;
use serde::{Deserialize, Serialize};
use std::{
    fs,
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
        .route("/_matrix/app/v1/transactions/:tx_id", put(handle_matrix_tx))
        .route("/login", get(login))
        .route("/submit", post(submit))
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

#[derive(Serialize, Deserialize, Default, Debug)]
struct AppConfig {
    client_id: String,
    client_secret: String,
}

#[derive(Serialize)]
struct AppDetails {
    client_name: String,
    redirect_uris: String,
}

async fn submit(body: String) -> String {
    let db = DB::open_default("data").unwrap();

    let mut parts = body.split("=");
    let server = parts.nth(1).unwrap().to_string();
    let config = db.get(&server).unwrap();
    create_app(&server).await;
    // println!("{:#?}", server);
    server
}

async fn create_app(server: &str) -> AppConfig {
    println!("{}", server);
    let mut url = Url::parse(server).unwrap();
    url.set_path("/api/v1/apps");
    // let query = format!("")
    let details = AppDetails {
        client_name: "Agora Social".to_string(),
        redirect_uris: "http://social.vera.pink/redirect".to_string(),
    };
    let ac = reqwest::Client::new()
        .post(url)
        .form(&details)
        .send()
        .await
        .unwrap()
        .json::<AppConfig>()
        .await
        .unwrap();
    println!("{:#?}", &ac);
    ac
}

async fn login() -> Html<String> {
    axum::response::Html(fs::read_to_string("login.html").unwrap())
}

#[debug_handler]
async fn handle_matrix_tx(
    data: Data<DatabaseHandle>,
    axum::extract::Path(tx_id): axum::extract::Path<String>,
    body: String,
) -> String {
    println!("in matrix handler");
    let tx: Transaction = serde_json::from_str(&body).unwrap();
    for event in tx.events {
        let re = Regex::new(r#".*\[\[(.*?)\]\].*"#).unwrap();
        let maybecaps = re.captures(&event.content.body);
        if let Some(caps) = maybecaps {
            println!("Found captures");
            let link = caps.get(1).unwrap().as_str();
            let mention = Mention {
                href: Url::parse("https://mastodon.loener.nl/users/v").unwrap(),
                kind: Default::default(),
            };
            let agora = format!("https://anagora.org/{}", link);
            let note = Note {
                kind: Default::default(),
                id: generate_object_id(DOMAIN).unwrap().into(),
                attributed_to: data.local_user().ap_id,
                to: vec![public()],
                content: format!(
                    "agora link -> [[<a href='{}'>{}</a>]] found in {}",
                    agora, link, body
                ),
                in_reply_to: None,
                tag: vec![mention],
            };
            CreatePost::send(
                note,
                Url::parse("https://mastodon.loener.nl/users/v/inbox").unwrap(),
                &data,
            )
            .await
            .unwrap();
        }
    }
    println!("{:#?}\n\nTransaction id: {}", body, tx_id);
    // let out = format!("Transaction: {:#?}", tx);
    body
}
