use crate::{
    database::DatabaseHandle,
    error::Error,
    objects::person::{DbUser, Person, PersonAcceptedActivities},
    webfinger::{build_webfinger_response, Webfinger},
};
use activitypub_federation::{
    axum::{
        inbox::{receive_activity, ActivityData},
        json::FederationJson,
    },
    config::Data,
    fetch::webfinger::extract_webfinger_name,
    protocol::context::WithContext,
    traits::Object,
};
use activitystreams_kinds::activity;
use axum::{
    extract::{Path, Query},
    response::{IntoResponse, Response},
    Json,
};
use axum_macros::debug_handler;
use http::StatusCode;
use serde::Deserialize;

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", self.0)).into_response()
    }
}

#[debug_handler]
pub async fn http_get_user(
    Path(name): Path<String>,
    data: Data<DatabaseHandle>,
) -> Result<FederationJson<WithContext<Person>>, Error> {
    let db_user = data.read_user(&name)?;
    let json_user = db_user.into_json(&data).await?;
    Ok(FederationJson(WithContext::new_default(json_user)))
}

#[debug_handler]
pub async fn http_post_user_inbox(
    data: Data<DatabaseHandle>,
    activity_data: ActivityData,
) -> impl IntoResponse {
    // let body = String::from(&activity_data.body);
    println!("INBOX: {}", String::from_utf8_lossy(&activity_data.body));
    receive_activity::<WithContext<PersonAcceptedActivities>, DbUser, DatabaseHandle>(
        activity_data,
        &data,
    )
    .await
}

#[derive(Deserialize)]
pub struct WebfingerQuery {
    resource: String,
}

#[debug_handler]
pub async fn webfinger(
    Query(query): Query<WebfingerQuery>,
    data: Data<DatabaseHandle>,
) -> Result<Json<Webfinger>, Error> {
    let name = extract_webfinger_name(&query.resource, &data).unwrap();
    println!("got webfinger {}", name);
    let db_user = data.read_user(&name)?;
    let resp = build_webfinger_response(query.resource, db_user).await;
    Ok(Json(resp))
}
