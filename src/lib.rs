#![feature(vec_resize_default)]
#![feature(try_from)]

use std::error::Error;
mod server;
mod coords;
mod game_grid;
mod client;
mod game_batch;
mod util;
#[cfg(feature = "mongodb_connector")] pub mod mongodb_connector;

pub use crate::game_batch::{GameBatch, SpecResult};
pub use crate::server::native::NativeServer;
#[cfg(feature = "js_server_connector")] pub use crate::server::js::JsServerWrapper;

pub type GameError = Box<dyn Error + Sync + Send>;