pub mod json_api;
mod json_server_wrapper;
mod native_server;

pub use self::json_server_wrapper::JsonServerWrapper;
pub use self::native_server::NativeServer;

use std::error::Error;
use coords::Coords;
use self::json_api::resp::ServerResponse;

pub trait GameServer {
	fn turn(
		&mut self,
		clear: Vec<Coords>,
		flag: Vec<Coords>,
		unflag: Vec<Coords>,
	) -> Result<(), Box<Error>>;

    fn status(&self) -> &ServerResponse;
}