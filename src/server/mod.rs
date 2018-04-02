mod json_api;
mod json_server_wrapper;
mod native_server;

pub use self::json_server_wrapper::JsonServerWrapper;
pub use self::native_server::NativeServer;

use std::error::Error;
use coords::Coords;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GameState { Ongoing, Win, Lose }

pub trait GameServer {
	fn turn(
		&mut self,
		clear: Vec<Coords>,
		flag: Vec<Coords>,
		unflag: Vec<Coords>,
	) -> Result<(), Box<Error>>;

	fn dims(&self) -> &[usize];

	fn mines(&self) -> usize;

	fn game_state(&self) -> GameState;

	fn cells_rem(&self) -> usize;

	fn clear_actual(&self) -> Vec<CellInfo>;
}

#[derive(Debug)]
pub struct CellInfo {
	pub coords: Coords,
	pub mine: bool,	
	pub surrounding: usize,
}