pub mod json_server_wrapper;
pub mod json_api;

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