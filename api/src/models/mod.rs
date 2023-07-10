use std::{collections::HashSet, sync::Mutex};
use tokio::sync::broadcast;

// Our shared state
pub struct AppState {
    // We require unique usernames. This tracks which usernames have been taken.
    pub user_set: Mutex<HashSet<String>>,
    // Channel used to send messages to all connected clients.
    pub tx: broadcast::Sender<String>,
}
