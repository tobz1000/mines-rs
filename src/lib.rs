#![feature(vec_resize_default)]
#![feature(iterator_try_fold)]
#![feature(try_from)]

#[macro_use] extern crate serde_derive;
#[macro_use] extern crate itertools;
#[macro_use] extern crate lazy_static;
#[cfg(test)] #[macro_use] extern crate quickcheck;

mod server;
mod coords;
mod game_grid;
mod client;
mod game_batch;
mod util;
#[cfg(feature = "mongodb_connector")] pub mod mongodb_connector;

use std::error::Error;

pub use game_batch::{GameBatch, SpecResult};
pub use server::native::NativeServer;
#[cfg(feature = "js_server_connector")] pub use server::js::JsServerWrapper;

pub type GameError = Box<dyn Error + Sync + Send>;