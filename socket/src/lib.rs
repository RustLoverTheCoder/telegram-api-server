mod server;
mod session;
mod socket;

pub use socket::CloseCode;
pub use socket::CloseFrame;
pub use socket::Message;
pub use socket::RawMessage;
pub use socket::Sink;
pub use socket::Socket;
pub use socket::Stream;

pub mod axum;

pub use server::{Server, ServerExt};
pub use session::{Session, SessionExt};

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Request = http::Request<()>;
