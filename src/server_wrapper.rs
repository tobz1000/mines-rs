use self::json_server_requests::*;
use self::server_response::ServerResponse;

trait MinesServer {
	fn status() -> ServerResponse;
	fn turn(
		clear:	&Vec<Vec<i32>>,
		flag:	&Vec<Vec<i32>>,
		unflag:	&Vec<Vec<i32>>,
		client:	Option<&str>
	) -> ServerResponse;
}

pub struct JsonServerWrapper<'a> {
	url: &'a str,
	client: &'a str,
	status: ServerResponse
}

// impl JsonServerWrapper<'a> {
// 	pub fn new_game(
// 		dims: &Vec<i32>,
// 		mines: i32,
// 		seed: Option<u64>
// 	) -> JsonServerWrapper {
// 		let req = NewGameRequest {

// 		}
// 	}
// }

mod json_server_requests {
	pub trait JsonServerRequest {
		const ACTION: &'static str;
	}

	#[derive(Serialize, Deserialize)]
	pub struct TurnRequest {
		id: Box<str>,
		client: Box<str>,
		clear: Vec<Vec<i32>>,
		flag: Vec<Vec<i32>>,
		unflag: Vec<Vec<i32>>,
	}

	impl JsonServerRequest for TurnRequest {
		const ACTION: &'static str = "turn";
	}

	#[derive(Serialize, Deserialize)]
	pub struct NewGameRequest {
		client: Box<str>,
		seed: Option<u32>,
		dims: Vec<i32>,
		mines: i32,
		autoclear: Option<bool>,
	}

	impl JsonServerRequest for NewGameRequest {
		const ACTION: &'static str = "new";
	}

	#[derive(Serialize, Deserialize)]
	pub struct StatusRequest {
		id: Box<str>
	}

	impl JsonServerRequest for StatusRequest {
		const ACTION: &'static str = "status";
	}
}

mod server_response {
	extern crate hyper_serde;
	extern crate time;

	use self::hyper_serde::Serde;
	use self::time::Tm;

	#[derive(Serialize, Deserialize)]
	enum CellState { Cleared, Mine }

	#[derive(Serialize, Deserialize)]
	struct CellInfo {
		surrounding: i32,
		state: CellState,
		coords: Vec<i32>
	}

	#[derive(Serialize, Deserialize)]
	#[allow(non_snake_case)]
	pub struct ServerResponse {
		id: String,
		seed: u64,
		dims: Vec<i32>,
		mines: i32,
		turnNum: i32,
		gameOver: bool,
		win: bool,
		cellsRem: i32,
		flagged: Vec<Vec<i32>>,
		unflagged: Vec<Vec<i32>>,
		clearActual: Vec<CellInfo>,
		clearReq: Vec<Vec<i32>>,
		turnTakenAt: Serde<Tm>
	}
}