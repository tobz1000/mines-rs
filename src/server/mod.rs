mod js;
mod native;

pub(crate) use self::js::JsServerWrapper;
pub(crate) use self::native::NativeServer;

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