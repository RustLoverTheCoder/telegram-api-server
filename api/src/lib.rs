pub mod extract;
pub mod handler;
pub mod models;

use axum::extract::Query;
use axum::response::IntoResponse;
use axum::Extension;
use axum::{response::Html, routing::get, Router};
use migration::{Migrator, MigratorTrait};
use sockets::axum::Upgrade;
use sqlx_core::sea_orm::Database;
use std::io::BufRead;
use std::str::FromStr;
use std::{env, net::SocketAddr};

use async_trait::async_trait;
use sockets::Error;
use sockets::Server;
use std::collections::HashMap;

type SessionID = u16;
type Session = sockets::Session<SessionID, ()>;

#[derive(Debug)]
enum ChatMessage {
    Send { from: SessionID, text: String },
}

struct ChatServer {
    sessions: HashMap<SessionID, Session>,
    handle: Server<Self>,
}

#[async_trait]
impl sockets::ServerExt for ChatServer {
    type Session = ChatSession;
    type Call = ChatMessage;

    async fn on_connect(
        &mut self,
        socket: sockets::Socket,
        _request: sockets::Request,
        _address: SocketAddr,
    ) -> Result<Session, Error> {
        let id = (0..).find(|i| !self.sessions.contains_key(i)).unwrap_or(0);
        let session = Session::create(
            |_| ChatSession {
                id,
                server: self.handle.clone(),
            },
            id,
            socket,
        );
        self.sessions.insert(id, session.clone());
        Ok(session)
    }

    async fn on_disconnect(
        &mut self,
        id: <Self::Session as sockets::SessionExt>::ID,
    ) -> Result<(), Error> {
        assert!(self.sessions.remove(&id).is_some());
        Ok(())
    }

    async fn on_call(&mut self, call: Self::Call) -> Result<(), Error> {
        match call {
            ChatMessage::Send { text, from } => {
                let sessions = self.sessions.iter().filter(|(id, _)| from != **id);
                let text = format!("from {from}: {text}");
                for (id, handle) in sessions {
                    tracing::info!("sending {text} to {id}");
                    handle.text(text.clone());
                }
            }
        };
        Ok(())
    }
}

struct ChatSession {
    id: SessionID,
    server: Server<ChatServer>,
}

#[async_trait]
impl sockets::SessionExt for ChatSession {
    type ID = SessionID;
    type Call = ();

    fn id(&self) -> &Self::ID {
        &self.id
    }
    async fn on_text(&mut self, text: String) -> Result<(), Error> {
        tracing::info!("received: {text}");
        self.server.call(ChatMessage::Send {
            from: self.id,
            text,
        });
        Ok(())
    }

    async fn on_binary(&mut self, _bytes: Vec<u8>) -> Result<(), Error> {
        unimplemented!()
    }

    async fn on_call(&mut self, call: Self::Call) -> Result<(), Error> {
        let () = call;
        Ok(())
    }
}

#[tokio::main]
pub async fn main() {
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

    let (server, _) = Server::create(|handle| ChatServer {
        sessions: HashMap::new(),
        handle,
    });

    let app = Router::new()
        .route("/", get(handler))
        .route("/apiws", get(websocket_handler))
        .layer(Extension(server.clone()));

    let address = SocketAddr::from_str(&server_url).unwrap();

    tokio::spawn(async move {
        tracing::debug!("listening on {}", address);
        axum::Server::bind(&address)
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .await
            .unwrap();
    });

    let stdin = std::io::stdin();
    let lines = stdin.lock().lines();
    for line in lines {
        let line = line.unwrap();
        server.call(ChatMessage::Send {
            text: line,
            from: SessionID::MAX, // reserve some ID for the server
        });
    }
}

async fn handler() -> Html<&'static str> {
    Html(std::include_str!("../chat.html"))
}

async fn websocket_handler(
    Extension(server): Extension<Server<ChatServer>>,
    Query(query): Query<HashMap<String, String>>,
    socket: Upgrade,
) -> impl IntoResponse {
    let kick_me = query.get("kick_me");
    let kick_me = kick_me.map(|s| s.as_str());
    if matches!(kick_me, Some("Yes")) {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            "we won't accept you because of kick_me query parameter",
        )
            .into_response();
    }
    socket.on_upgrade(server)
}