#![feature(derive_default_enum)]

#[macro_use]
extern crate proc_macros;

#[macro_use]
extern crate log;

mod requests;
mod websocket_handle;

pub use websocket_handle::Connection;

mod discord_structs;
pub use discord_structs::*;

pub use requests::DiscordClient;
pub use requests::NetError;

pub use futures_util::StreamExt;
pub extern crate serde_json;
pub use tokio_tungstenite::tungstenite::Message;
pub extern crate async_channel;
pub extern crate hyper;
