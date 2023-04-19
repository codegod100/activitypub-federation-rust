//! edgedb functions
use serde::Deserialize;

use crate::error::Error;

#[derive(Deserialize, Debug, Clone)]
///database
pub struct Link {
    ///database
    pub rel: String,
    #[serde(rename = "type")]
    ///database
    pub kind: String,
    ///database
    pub href: String,
}
#[derive(Deserialize, Debug)]
/// edgedb person
pub struct Person {
    ///database
    pub id: String,
    ///database field
    pub username: String,
    ///database field
    pub links: Vec<Link>,
}
///return people from database
pub async fn people() -> Result<Vec<Person>, anyhow::Error> {
    let conn = edgedb_tokio::create_client().await?;
    let va = conn
        .query_json("select Person{id,username,links}", &())
        .await?;
    let people: Vec<Person> = serde_json::from_str(&va.to_string()).unwrap();
    Ok(people)
}

pub async fn find_person(name: &str) -> Result<Person, anyhow::Error> {
    let conn = edgedb_tokio::create_client().await?;
    let va = conn
        .query_single_json(
            "select Person{id,username,links} filter .username = $0",
            &(name, ""),
        )
        .await?
        .unwrap();
    let person: Person = serde_json::from_str(&va.to_string()).unwrap();
    Ok(person)
}
