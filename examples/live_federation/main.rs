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
    extract::{Json, Query},
    response::{Html, Redirect},
    routing::{get, post, put},
    Router,
};
use edgedb_protocol::{
    codec::{NamedTuple, ObjectShape, Set},
    model::Uuid,
};
use error::Error;
use regex::Regex;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    net::ToSocketAddrs,
    result,
    sync::{Arc, Mutex},
};
use tracing::log::{info, LevelFilter};

mod activities;
mod database;
mod edge;
mod error;
#[allow(clippy::diverging_sub_expression, clippy::items_after_statements)]
mod http;
mod objects;
mod utils;
mod webfinger;

const DOMAIN: &str = "social.vera.pink";
const LOCAL_USER_NAME: &str = "v";
const BIND_ADDRESS: &str = "0.0.0.0:8003";

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
    let people = edge::people().await.unwrap();
    let mut users = vec![];
    for person in people {
        let user = DbUser::new(DOMAIN, &person)?;
        users.push(user);
    }
    let database = Arc::new(Database {
        users: Mutex::new(users),
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
        // .route("/submit", post(submit))
        // .route("/redirect", get(auth))
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
    // redirect_uri: String,
}

#[derive(Serialize)]
struct AppDetails {
    client_name: String,
    redirect_uris: String,
}

// async fn credentials(server: &str) -> AppConfig {
//     let db = DB::open_default("data").unwrap();

//     let config = db.get(&server).unwrap();
//     let ac = match config {
//         Some(chars) => {
//             println!("got oauth app details");
//             let data: AppConfig = serde_json::from_slice(&chars).unwrap();
//             data
//         }
//         None => {
//             println!("creating new oauth app");
//             let ac = create_app(&server).await;
//             let data = serde_json::to_vec(&ac).unwrap();
//             db.put(&server, data).unwrap();
//             ac
//         }
//     };
//     ac
// }
#[derive(Deserialize, Debug)]
struct Token {
    access_token: String,
}

struct User {
    url: String,
}

// async fn auth(Query(params): Query<HashMap<String, String>>) -> String {
//     let server = &params["server"];
//     let code = &params["code"];
//     println!("code: {}; server {}", code, server);
//     let mut client = reqwest::Client::new();
//     // let ac = credentials(server).await;
//     println!("credentials: {:#?}", ac);
//     let mut params = HashMap::new();
//     params.insert("client_id", ac.client_id);
//     params.insert("client_secret", ac.client_secret);
//     params.insert(
//         "redirect_uri",
//         format!("https://social.vera.pink/redirect?server={}", server),
//     );
//     params.insert("grant_type", "authorization_code".to_string());
//     params.insert("code", code.to_owned());
//     params.insert("force_login", "true".to_string());
//     println!("params: {:#?}", params);

//     let token_url = format!("{}/oauth/token", server);
//     let token = client
//         .post(token_url)
//         .form(&params)
//         .send()
//         .await
//         .unwrap()
//         .json::<Token>()
//         .await
//         .unwrap();
//     println!("access token: {:#?}", token);
//     client = reqwest::Client::new();
//     let url = format!("{}/api/v1/accounts/verify_credentials", server);
//     let res = client
//         .get(url)
//         .header("Authorization", format!("Bearer {}", token.access_token))
//         .send()
//         .await
//         .unwrap()
//         .text()
//         .await
//         .unwrap();
//     println!("USER: {}", res);
//     res
// }
// async fn submit(body: String) -> Redirect {
//     let mut parts = body.split("=");
//     let server = decode(parts.nth(1).unwrap()).unwrap().to_string();
//     // println!("{:#?}", server);
//     let redirect = format!("https://social.vera.pink/redirect");
//     let ac = credentials(&server).await;
//     println!("Credentials: {:#?}", ac);
//     Redirect::to(&format!(
//         "{}/oauth/authorize?response_type=code&client_id={}&redirect_uri={}?server={}",
//         server, ac.client_id, redirect, server
//     ))
// }

async fn create_app(server: &str) -> AppConfig {
    println!("{}", server);
    let mut url = Url::parse(server).unwrap();
    url.set_path("/api/v1/apps");
    // let query = format!("")
    let details = AppDetails {
        client_name: "Agora Social".to_string(),
        redirect_uris: "https://social.vera.pink/redirect".to_string(),
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
                attributed_to: data.local_user("v").ap_id,
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
