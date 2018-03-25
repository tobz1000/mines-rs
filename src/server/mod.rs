pub mod json_server_wrapper;

use std::error::Error;
use game_grid::Coords;
use self::json_server_wrapper::server_response::ServerResponse;

pub trait GameServer {
	fn turn(
		&mut self,
		clear: Vec<Coords>,
		flag: Vec<Coords>,
		unflag: Vec<Coords>,
	) -> Result<(), Box<Error>>;

    fn status(&self) -> &ServerResponse;
}