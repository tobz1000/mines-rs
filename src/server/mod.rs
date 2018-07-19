mod json_api;
mod json_server_wrapper;
mod native_server;
mod db;

pub(crate) use self::json_server_wrapper::JsonServerWrapper;
pub(crate) use self::native_server::NativeServer;

use ::GameError;
use coords::Coords;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GameState { Ongoing, Win, Lose }

pub trait GameServer {
	fn turn(
		&mut self,
		clear: Vec<Coords>,
		flag: Vec<Coords>,
		unflag: Vec<Coords>,
	) -> Result<Vec<CellInfo>, GameError>;

	fn dims(&self) -> &[usize];

	fn mines(&self) -> usize;

	fn game_state(&self) -> GameState;

	fn cells_rem(&self) -> usize;
}

#[derive(Debug)]
pub struct CellInfo {
	pub coords: Coords,
	pub mine: bool,
	pub surrounding: usize,
}