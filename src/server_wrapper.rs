extern crate tokio_core;
extern crate hyper;
extern crate futures;
extern crate serde;
extern crate serde_json;

use std::error::Error;
use self::tokio_core::reactor;
use self::futures::{Future, Stream};
use self::hyper::Uri;
use self::hyper::Method;
use self::hyper::client::{Client, HttpConnector, Request};
use self::hyper::header::{ContentLength, ContentType};
use self::serde::ser::Serialize;

use self::json_server_requests::*;
use self::server_response::ServerResponse;

pub trait MinesServer {
	fn status(&self) -> &ServerResponse;
	fn turn(
		&self,
		clear:	Vec<Vec<i32>>,
		flag:	Vec<Vec<i32>>,
		unflag:	Vec<Vec<i32>>,
	) -> Result<ServerResponse, Box<Error>>;
}

pub struct JsonServerWrapper<'a> {
	base_url: &'a str,
	client_name: &'a str,
	status: ServerResponse,
	http_client: Client<HttpConnector>,
}

impl<'a> JsonServerWrapper<'a> {
	pub fn new_game(
		dims: Vec<i32>,
		mines: i32,
		seed: Option<u32>
	) -> Result<JsonServerWrapper<'a>, Box<Error>> {
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
			base_url,
			client_name,
			status,
			http_client
		})
	}

	fn action<R: JsonServerRequest + Serialize>(&self, request: R) ->
		Result<ServerResponse, Box<Error>>
	{
		Self::_action(&self.http_client, &self.base_url, &request)
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
}

impl<'a> MinesServer for JsonServerWrapper<'a> {
	fn turn(
		&self,
		clear: Vec<Vec<i32>>,
		flag: Vec<Vec<i32>>,
		unflag: Vec<Vec<i32>>
	) -> Result<ServerResponse, Box<Error>> {
		self.action(TurnRequest {
			id: &self.status.id,
			client: &self.client_name,
			clear,
			flag,
			unflag			
		})
	}

	fn status(&self) -> &ServerResponse {
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