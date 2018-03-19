extern crate tokio_core;
extern crate hyper;
extern crate serde;
extern crate serde_json;
extern crate futures_await as futures;

use std::error::Error;
use std::io;
use std::str;
use game_grid::Coords;
use self::tokio_core::reactor;
use self::futures::{Future, Stream};
use self::hyper::Method;
use self::hyper::client::{Client, HttpConnector, Request};
use self::hyper::header::{ContentLength, ContentType};
use self::serde::ser::Serialize;

use self::json_server_requests::*;
use self::server_response::ServerResponse;

pub struct JsonServerWrapper {
	client_name: String,
	status: ServerResponse,
	base_url: String,
	http_client: Client<HttpConnector>
}

impl JsonServerWrapper {
	pub fn new_game(
		dims: Coords,
		mines: usize,
		seed: Option<u64>,
		event_loop_core: &mut reactor::Core
	) -> Result<JsonServerWrapper, Box<Error>> {
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
				autoclear: true
			},
			&http_client,
			event_loop_core,
		)?;

		Ok(JsonServerWrapper {
			base_url: base_url.to_owned(),
			client_name: client_name.to_owned(),
			status,
			http_client
		})
	}

	fn action<R: JsonServerRequest + Serialize>(
		self,
		request: R,
		event_loop_core: &mut reactor::Core,
	) -> Result<JsonServerWrapper, Box<Error>> {
		let status = Self::_action(
			&self.base_url,
			&request,
			&self.http_client,
			event_loop_core,
		)?;

		Ok(JsonServerWrapper { status, ..self })
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

	pub fn turn(
		self,
		clear: Vec<Coords>,
		flag: Vec<Coords>,
		unflag: Vec<Coords>,
		event_loop_core: &mut reactor::Core
	) -> Result<JsonServerWrapper, Box<Error>> {
		let req = TurnRequest {
			id: &self.status.id.clone(),
			client: &self.client_name.clone(),
			clear,
			flag,
			unflag
		};

		self.action(req, event_loop_core)
	}

	pub fn status(&self) -> &ServerResponse {
		&self.status
	}
}

mod json_server_requests {
	use game_grid::Coords;

	pub trait JsonServerRequest {
		const ACTION: &'static str;
	}

	#[derive(Serialize, Deserialize)]
	pub struct TurnRequest<'a> {
		pub id: &'a str,
		pub client: &'a str,
		pub clear: Vec<Coords>,
		pub flag: Vec<Coords>,
		pub unflag: Vec<Coords>,
	}

	impl<'a> JsonServerRequest for TurnRequest<'a> {
		const ACTION: &'static str = "turn";
	}

	#[derive(Serialize, Deserialize)]
	pub struct NewGameRequest<'a> {
		pub client: &'a str,
		pub seed: Option<u64>,
		pub dims: Coords,
		pub mines: usize,
		pub autoclear: bool,
	}

	impl<'a> JsonServerRequest for NewGameRequest<'a> {
		const ACTION: &'static str = "new";
	}

	#[derive(Serialize, Deserialize)]
	pub struct StatusRequest<'a> {
		pub id: &'a str
	}

	impl<'a> JsonServerRequest for StatusRequest<'a> {
		const ACTION: &'static str = "status";
	}
}

pub mod server_response {
	extern crate chrono;

	use self::chrono::{DateTime, Utc};
	use game_grid::Coords;

	#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
	#[serde(rename_all = "camelCase")]
	pub enum CellState { Cleared, Mine }

	#[derive(Debug, Serialize, Deserialize)]
	#[serde(rename_all = "camelCase")]
	pub struct CellInfo {
		pub surrounding: usize,
		pub state: CellState,
		pub coords: Coords
	}

	#[derive(Debug, Serialize, Deserialize)]
	#[serde(rename_all = "camelCase")]
	pub struct ServerResponse {
		pub id: String,
		pub seed: u64,
		pub dims: Coords,
		pub mines: usize,
		pub turn_num: i32,
		pub game_over: bool,
		pub win: bool,
		pub cells_rem: i32,
		pub flagged: Vec<Coords>,
		pub unflagged: Vec<Coords>,
		pub clear_actual: Vec<CellInfo>,
		pub clear_req: Vec<Coords>,
		pub turn_taken_at: DateTime<Utc>,
	}
}