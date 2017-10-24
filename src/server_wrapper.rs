extern crate tokio_core;
extern crate hyper;
extern crate futures;
extern crate serde;
extern crate serde_json;

use std::error::Error;
use std::io;
use self::tokio_core::reactor;
use self::futures::{Future, Stream};
use self::hyper::Method;
use self::hyper::client::{Client, HttpConnector, Request};
use self::hyper::header::{ContentLength, ContentType};
use self::serde::ser::Serialize;

use self::json_server_requests::*;
use self::server_response::ServerResponse;

pub struct JsonServerWrapper {
	base_url: String,
	client_name: String,
	status: ServerResponse,
	http_client: Client<HttpConnector>
}

impl JsonServerWrapper {
	pub fn new_game(
		dims: Vec<i32>,
		mines: i32,
		seed: Option<u32>,
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
				autoclear: false
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
		clear: Vec<Vec<i32>>,
		flag: Vec<Vec<i32>>,
		unflag: Vec<Vec<i32>>,
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
	pub trait JsonServerRequest {
		const ACTION: &'static str;
	}

	#[derive(Serialize, Deserialize)]
	pub struct TurnRequest<'a> {
		pub id: &'a str,
		pub client: &'a str,
		pub clear: Vec<Vec<i32>>,
		pub flag: Vec<Vec<i32>>,
		pub unflag: Vec<Vec<i32>>,
	}

	impl<'a> JsonServerRequest for TurnRequest<'a> {
		const ACTION: &'static str = "turn";
	}

	#[derive(Serialize, Deserialize)]
	pub struct NewGameRequest<'a> {
		pub client: &'a str,
		pub seed: Option<u32>,
		pub dims: Vec<i32>,
		pub mines: i32,
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

mod server_response {
	extern crate chrono;

	use self::chrono::{DateTime, Utc};

	#[derive(Serialize, Deserialize)]
	pub enum CellState { Cleared, Mine }

	#[derive(Serialize, Deserialize)]
	pub struct CellInfo {
		surrounding: i32,
		state: CellState,
		coords: Vec<i32>
	}

	#[derive(Serialize, Deserialize)]
	#[allow(non_snake_case)]
	pub struct ServerResponse {
		pub id: String,
		pub seed: u64,
		pub dims: Vec<i32>,
		pub mines: i32,
		pub turnNum: i32,
		pub gameOver: bool,
		pub win: bool,
		pub cellsRem: i32,
		pub flagged: Vec<Vec<i32>>,
		pub unflagged: Vec<Vec<i32>>,
		pub clearActual: Vec<CellInfo>,
		pub clearReq: Vec<Vec<i32>>,
		pub turnTakenAt: DateTime<Utc>,
	}
}