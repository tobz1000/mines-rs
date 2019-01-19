#![feature(vec_resize_default)]
#![feature(try_from)]

use std::error::Error;
mod client;
mod coords;
mod game_batch;
mod game_grid;
#[cfg(feature = "mongodb_connector")]
pub mod mongodb_connector;
mod server;
mod util;

pub use crate::game_batch::{GameBatch, SpecResult};
#[cfg(feature = "js_server_connector")]
pub use crate::server::js::JsServerWrapper;
pub use crate::server::native::NativeServer;

pub type GameError = Box<dyn Error + Sync + Send>;
