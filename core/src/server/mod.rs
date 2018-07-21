mod js;
mod native;

pub use self::js::JsServerWrapper;
pub use self::native::{NativeServer, NativeServerConfig};

use ::GameError;
use coords::Coords;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GameState { Ongoing, Win, Lose }

pub trait GameServer: Sized {
	type Config: Clone + Send + Sync;

	fn new(
		dims: Vec<usize>,
		mines: usize,
		seed: Option<u32>,
		autoclear: bool,
		config: Self::Config
	) -> Result<Self, GameError>;

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