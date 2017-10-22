extern crate tokio_core;
extern crate hyper;
extern crate futures;
extern crate serde;
extern crate serde_json;

use std::error::Error;
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
	http_client: Client<HttpConnector>,
}

impl JsonServerWrapper {
	pub fn new_game(
		dims: Vec<i32>,
		mines: i32,
		seed: Option<u32>
	) -> Result<JsonServerWrapper, Box<Error>> {
		let client_name = "RustyBoi";
		let event_loop_core = reactor::Core::new()?;
		let http_client = Client::new(&event_loop_core.handle());
		let base_url = "http://localhost:1066/server/";

		let status = Self::_action(
			&http_client,
			&base_url,
			&NewGameRequest {
				client: client_name,
				seed,
				dims,
				mines,
				autoclear: false
			}
		)?;

		Ok(JsonServerWrapper {
			base_url: base_url.to_owned(),
			client_name: client_name.to_owned(),
			status,
			http_client
		})
	}

	fn action<R: JsonServerRequest + Serialize>(self, request: R) ->
		Result<JsonServerWrapper, Box<Error>>
	{
		let status = Self::_action(
			&self.http_client,
			&self.base_url,
			&request
		)?;

		Ok(JsonServerWrapper { status, ..self })
	}

	fn _action<R: JsonServerRequest + Serialize>(
		http_client: &Client<HttpConnector>,
		base_url: &str,
		request: &R
	) -> Result<ServerResponse, Box<Error>> {
		let post_url = format!("{}/{}", base_url, R::ACTION).parse()?;
		let req_json = serde_json::to_string(&request)?;

		let mut http_req = Request::new(Method::Post, post_url);
		http_req.headers_mut().set(ContentType::json());
		http_req.headers_mut().set(ContentLength(req_json.len() as u64));
		http_req.set_body(req_json);

		let http_resp = http_client.request(http_req).wait()?;
		let body = http_resp.body().concat2().wait()?;
		let server_resp = serde_json::from_slice(&body)?;

		Ok(server_resp)
	}

	pub fn turn(
		self,
		clear: Vec<Vec<i32>>,
		flag: Vec<Vec<i32>>,
		unflag: Vec<Vec<i32>>
	) -> Result<JsonServerWrapper, Box<Error>> {
		let req = TurnRequest {
			id: &self.status.id.clone(),
			client: &self.client_name.clone(),
			clear,
			flag,
			unflag			
		};

		self.action(req)
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
	extern crate hyper_serde;
	extern crate time;

	use self::hyper_serde::Serde;
	use self::time::Tm;

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
		pub turnTakenAt: Serde<Tm>
	}
}