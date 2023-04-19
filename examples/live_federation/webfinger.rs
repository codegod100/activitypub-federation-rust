use std::collections::HashMap;

use activitypub_federation::FEDERATION_CONTENT_TYPE;
use reqwest::Url;
use serde::{Deserialize, Serialize};

use crate::{edge, objects::person::DbUser};

/// A webfinger response with information about a `Person` or other type of actor.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Webfinger {
    /// The actor which is described here, for example `acct:LemmyDev@mastodon.social`
    pub subject: String,
    /// Links where further data about `subject` can be retrieved
    pub links: Vec<WebfingerLink>,
    /// Other Urls which identify the same actor as the `subject`
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<Url>,
    /// Additional data about the subject
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub properties: HashMap<Url, String>,
}

/// A single link included as part of a [Webfinger] response.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct WebfingerLink {
    /// Relationship of the link, such as `self` or `http://webfinger.net/rel/profile-page`
    pub rel: Option<String>,
    /// Media type of the target resource
    #[serde(rename = "type")]
    pub kind: Option<String>,
    /// Url pointing to the target resource
    pub href: Option<Url>,
    /// Additional data about the link
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub properties: HashMap<Url, String>,
}

pub async fn build_webfinger_response(subject: String, user: DbUser) -> Webfinger {
    let url = user.ap_id.into_inner();
    Webfinger {
        subject,
        links: vec![
            WebfingerLink {
                rel: Some("http://webfinger.net/rel/profile-page".to_string()),
                kind: Some("text/html".to_string()),
                href: Some(url.clone()),
                properties: Default::default(),
            },
            WebfingerLink {
                rel: Some("self".to_string()),
                kind: Some(FEDERATION_CONTENT_TYPE.to_string()),
                href: Some(url),
                properties: Default::default(),
            },
            WebfingerLink {
                rel: Some("git".to_string()),
                kind: Some("vcs-git".to_string()),
                href: Some(
                    Url::parse("https://github.com/codegod100/activitypub-federation-rust")
                        .unwrap(),
                ),
                properties: Default::default(),
            },
        ],
        aliases: vec![],
        properties: Default::default(),
    }
}
