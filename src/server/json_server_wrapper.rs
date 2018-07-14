extern crate hyper_sync;
extern crate serde;
extern crate serde_json;

use std::str;
use coords::Coords;
use server::{GameServer, GameState, CellInfo as NativeCellInfo};
use server::json_api::req::{JsonServerRequest, TurnRequest, NewGameRequest};
use server::json_api::resp::{ServerResponse, CellState, CellInfo as JsonCellInfo};
use ::GameError;
use self::hyper_sync::Client;
use self::hyper_sync::header::{ContentLength, ContentType};
use self::serde::ser::Serialize;

pub struct JsonServerWrapper {
	client_name: String,
	status: ServerResponse,
	base_url: String,
	http_client: Client,
}

impl JsonServerWrapper {
	pub fn new_game(
		dims: Vec<usize>,
		mines: usize,
		seed: Option<u32>,
		autoclear: bool,
	) -> Result<JsonServerWrapper, GameError> {
		let client_name = "RustyBoi";
		let http_client = Client::new();
		let base_url = "http://localhost:1066/server";

		let status = Self::_action(
			&base_url,
			&NewGameRequest {
				client: client_name,
				seed,
				dims,
				mines,
				autoclear,
			},
			&http_client,
		)?;

		Ok(JsonServerWrapper {
			base_url: base_url.to_owned(),
			client_name: client_name.to_owned(),
			status,
			http_client,
		})
	}

	fn action<R: JsonServerRequest + Serialize>(
		&mut self,
		request: R,
	) -> Result<(), GameError> {
		self.status = Self::_action(
			&self.base_url,
			&request,
			&self.http_client,
		)?;

		Ok(())
	}

	fn _action<R: JsonServerRequest + Serialize>(
		base_url: &str,
		request: &R,
		http_client: &Client,
	) -> Result<ServerResponse, GameError> {
		let post_url = format!("{}/{}", base_url, R::ACTION);
		let req_json = serde_json::to_string(&request)?;

		let http_req = http_client.post(&post_url)
			.header(ContentType::json())
			.header(ContentLength(req_json.len() as u64))
			.body(&req_json);

		let resp_buffer = http_req.send()?;

		let resp: ServerResponse = serde_json::from_reader(resp_buffer)?;
		Ok(resp)
	}

	fn turn(
		&mut self,
		clear: Vec<Coords>,
		flag: Vec<Coords>,
		unflag: Vec<Coords>
	) -> Result<(), GameError> {
		let req = TurnRequest {
			id: &self.status.id.clone(),
			client: &self.client_name.clone(),
			clear,
			flag,
			unflag
		};

		self.action(req)
	}
}

impl<'a> GameServer for JsonServerWrapper {
	fn turn(
		&mut self,
		clear: Vec<Coords>,
		flag: Vec<Coords>,
		unflag: Vec<Coords>,
	) -> Result<Vec<NativeCellInfo>, GameError> {
		self.turn(clear, flag, unflag)?;

		let clear_actual_native = self.status.clear_actual.iter()
			.map(|&JsonCellInfo {
				surrounding,
				state,
				ref coords
			}| NativeCellInfo {
				coords: coords.clone(),
				mine: state == CellState::Mine,
				surrounding
			})
			.collect();

		Ok(clear_actual_native)
	}

	fn dims(&self) -> &[usize] { &self.status.dims }

	fn mines(&self) -> usize { self.status.mines }

	fn game_state(&self) -> GameState {
		if self.status.game_over {
			if self.status.win {
				GameState::Win
			} else {
				GameState::Lose
			}
		} else {
			GameState::Ongoing
		}
	}

	fn cells_rem(&self) -> usize { self.status.cells_rem }
}