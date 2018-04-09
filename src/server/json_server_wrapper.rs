extern crate tokio_core;
extern crate hyper;
extern crate serde;
extern crate serde_json;
extern crate futures_await as futures;

use std::error::Error;
use std::io;
use std::str;
use coords::Coords;
use server::{GameServer, GameState, CellInfo as NativeCellInfo};
use server::json_api::req::{JsonServerRequest, TurnRequest, NewGameRequest};
use server::json_api::resp::{ServerResponse, CellState, CellInfo as JsonCellInfo};
use self::tokio_core::reactor;
use self::futures::{Future, Stream};
use self::hyper::Method;
use self::hyper::client::{Client, HttpConnector, Request};
use self::hyper::header::{ContentLength, ContentType};
use self::serde::ser::Serialize;

pub struct JsonServerWrapper<'a> {
	client_name: String,
	status: ServerResponse,
	base_url: String,
	http_client: Client<HttpConnector>,
	event_loop_core: &'a mut reactor::Core
}

impl<'a> JsonServerWrapper<'a> {
	pub fn new_game(
		dims: Vec<usize>,
		mines: usize,
		seed: Option<u32>,
		autoclear: bool,
		event_loop_core: &'a mut reactor::Core
	) -> Result<JsonServerWrapper<'a>, Box<Error>> {
		let client_name = "RustyBoi";
		let http_client = Client::new(&event_loop_core.handle());
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
			event_loop_core,
		)?;

		Ok(JsonServerWrapper {
			base_url: base_url.to_owned(),
			client_name: client_name.to_owned(),
			status,
			http_client,
			event_loop_core
		})
	}

	fn action<R: JsonServerRequest + Serialize>(
		&mut self,
		request: R,
	) -> Result<(), Box<Error>> {
		self.status = Self::_action(
			&self.base_url,
			&request,
			&self.http_client,
			self.event_loop_core,
		)?;

		Ok(())
	}

	fn _action<R: JsonServerRequest + Serialize>(
		base_url: &str,
		request: &R,
		http_client: &Client<HttpConnector>,
		event_loop_core: &mut reactor::Core,
	) -> Result<ServerResponse, Box<Error>> {
		let post_url = format!("{}/{}", base_url, R::ACTION).parse()?;
		let req_json = serde_json::to_string(&request)?;

		let mut http_req = Request::new(Method::Post, post_url);
		http_req.headers_mut().set(ContentType::json());
		http_req.headers_mut().set(ContentLength(req_json.len() as u64));
		http_req.set_body(req_json);

		let server_resp_fut = http_client.request(http_req).and_then(|resp| {
			resp.body().concat2().and_then(|body| {
				Ok(serde_json::from_slice(&body).map_err(|e| {
					io::Error::new(io::ErrorKind::InvalidData, e)
				})?)
			})
		});

		Ok(event_loop_core.run(server_resp_fut)?)
	}

	fn turn(
		&mut self,
		clear: Vec<Coords>,
		flag: Vec<Coords>,
		unflag: Vec<Coords>
	) -> Result<(), Box<Error>> {
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

impl<'a> GameServer for JsonServerWrapper<'a> {
	fn turn(
		&mut self,
		clear: Vec<Coords>,
		flag: Vec<Coords>,
		unflag: Vec<Coords>,
	) -> Result<Vec<NativeCellInfo>, Box<Error>> {
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