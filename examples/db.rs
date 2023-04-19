use activitypub_federation::edge::people;

#[tokio::main]
async fn main() {
    let db = people().await.unwrap();
    println!("{:#?}", db);
}
