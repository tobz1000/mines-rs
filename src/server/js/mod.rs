mod api;

use api::req::{JsServerRequest, NewGameRequest, TurnRequest};
use api::resp::{CellInfo as JsCellInfo, CellState, ServerResponse};
use hyper_sync::header::{ContentLength, ContentType};
use hyper_sync::Client;
use serde::ser::Serialize;
use std::str;

use crate::coords::Coords;
use crate::server::{CellInfo as NativeCellInfo, GameServer, GameSpec, GameState};
use crate::GameError;

pub struct JsServerWrapper {
    client_name: String,
    status: ServerResponse,
    base_url: String,
    http_client: Client,
}

impl JsServerWrapper {
    pub fn new(
        GameSpec {
            dims,
            mines,
            seed,
            autoclear,
        }: GameSpec,
    ) -> Result<JsServerWrapper, GameError> {
        let client_name = "RustyBoi";
        let http_client = Client::new();
        let base_url = "http://localhost:1066/server";

        let status = Self::_action(
            &base_url,
            &NewGameRequest {
                client: client_name,
                seed: Some(seed),
                dims,
                mines,
                autoclear,
            },
            &http_client,
        )?;

        Ok(JsServerWrapper {
            base_url: base_url.to_owned(),
            client_name: client_name.to_owned(),
            status,
            http_client,
        })
    }

    fn action<R: JsServerRequest + Serialize>(&mut self, request: R) -> Result<(), GameError> {
        self.status = Self::_action(&self.base_url, &request, &self.http_client)?;

        Ok(())
    }

    fn _action<R: JsServerRequest + Serialize>(
        base_url: &str,
        request: &R,
        http_client: &Client,
    ) -> Result<ServerResponse, GameError> {
        let post_url = format!("{}/{}", base_url, R::ACTION);
        let req_json = serde_json::to_string(&request)?;

        let http_req = http_client
            .post(&post_url)
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
        unflag: Vec<Coords>,
    ) -> Result<(), GameError> {
        let req = TurnRequest {
            id: &self.status.id.clone(),
            client: &self.client_name.clone(),
            clear,
            flag,
            unflag,
        };

        self.action(req)
    }
}

impl<'a> GameServer for JsServerWrapper {
    fn turn(
        &mut self,
        clear: Vec<Coords>,
        flag: Vec<Coords>,
        unflag: Vec<Coords>,
    ) -> Result<Vec<NativeCellInfo>, GameError> {
        self.turn(clear, flag, unflag)?;

        let clear_actual_native = self
            .status
            .clear_actual
            .iter()
            .map(
                |&JsCellInfo {
                     surrounding,
                     state,
                     ref coords,
                 }| NativeCellInfo {
                    coords: coords.clone(),
                    mine: state == CellState::Mine,
                    surrounding,
                },
            )
            .collect();

        Ok(clear_actual_native)
    }

    fn dims(&self) -> &[usize] {
        &self.status.dims
    }

    fn mines(&self) -> usize {
        self.status.mines
    }

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

    fn cells_rem(&self) -> usize {
        self.status.cells_rem
    }
}
