mod js;
mod native;

pub use self::js::JsServerWrapper;
pub use self::native::{
	NativeServer,
	DbInserter,
	MongoDbInserter,
	MemDbInserter
};

use ::GameError;
use coords::Coords;

#[derive(Debug)]
pub struct GameSpec {
    pub dims: Vec<usize>,
    pub mines: usize,
    pub seed: u32,
    pub autoclear: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GameState { Ongoing, Win, Lose }

pub trait GameServer: Sized {
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