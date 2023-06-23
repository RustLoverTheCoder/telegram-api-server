pub mod extract;
pub mod handler;

use axum::{response::Html, routing::get, Router, Server};
use core::sea_orm::Database;
use handler::websocket::{websocket_handler, AppState};
use migration::{Migrator, MigratorTrait};
use std::str::FromStr;
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};
use std::{env, net::SocketAddr};
use tokio::sync::broadcast;

#[tokio::main]
async fn start() -> anyhow::Result<()> {
    env::set_var("RUST_LOG", "debug");
    tracing_subscriber::fmt::init();

    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let host = env::var("HOST").expect("HOST is not set in .env file");
    let port = env::var("PORT").expect("PORT is not set in .env file");
    let server_url = format!("{host}:{port}");

    let conn = Database::connect(db_url)
        .await
        .expect("Database connection failed");
    Migrator::up(&conn, None).await.unwrap();

    let user_set = Mutex::new(HashSet::new());
    let (tx, _rx) = broadcast::channel(100);
    let app_state = Arc::new(AppState { user_set, tx });

    let app = Router::new()
        .route("/", get(handler))
        .route("/apiws", get(websocket_handler))
        .with_state(app_state);

    let addr = SocketAddr::from_str(&server_url).unwrap();
    Server::bind(&addr).serve(app.into_make_service()).await?;

    Ok(())
}

async fn handler() -> Html<&'static str> {
    Html(std::include_str!("../chat.html"))
}

pub fn main() {
    let result = start();

    if let Some(err) = result.err() {
        println!("Error: {err}");
    }
}
