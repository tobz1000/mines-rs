use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use mongodb::db::{Database, ThreadedDatabase};
use mongodb::oid::ObjectId;
use serde_derive::{Deserialize, Serialize};
use wither::Model;

use crate::server::GameState;
use crate::server::native::{NativeServer, TurnInfo, Cell, CellAction};
use crate::coords::Coords;

lazy_static! {
    static ref DB_CONNECTION: Database = {
        let client = mongodb::ThreadedClient::connect("localhost", 27017).unwrap();
        Database::open(client, "test", None, None)
    };
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "camelCase")]
enum CellState { Empty, Cleared, Mine }

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CellInfo {
    surrounding: i32,
    state: CellState,
    coords: Coords<i32>
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Game {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    created_at: DateTime<Utc>,
    pass: Option<String>,
    seed: i32,
    dims: Vec<i32>,
    size: i32,
    mines: i32,
    autoclear: bool,
    turns: Option<Vec<Turn>>,
    clients: Vec<String>,
    cell_array: Vec<CellState>,
    flag_array: Vec<bool>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Turn {
    turn_taken_at: DateTime<Utc>,
    clear_req: Vec<Coords<i32>>,
    clear_actual: Vec<CellInfo>,
    flagged: Vec<Coords<i32>>,
    unflagged: Vec<Coords<i32>>,
    game_over: bool,
    win: bool,
    cells_rem: i32,
}

impl<'a> Model<'a> for Game {
    const COLLECTION_NAME: &'static str = "games";

    fn id(&self) -> Option<ObjectId> {
        self.id.clone()
    }

    fn set_id(&mut self, oid: ObjectId) {
        self.id = Some(oid);
    }
}

impl Game {
    fn from_native(server: &NativeServer) -> Self {
        let &NativeServer {
            created_at,
            ref dims,
            ref grid,
            mines,
            seed,
            autoclear,
            turns: ref native_turns,
            ..
        } = server;

        let cell_array = grid.iter().map(|&Cell { mine, action, .. }| {
            match (mine, action) {
                (true, _) => CellState::Mine,
                (false, CellAction::Cleared) => CellState::Cleared,
                (false, _) => CellState::Empty
            }
        }).collect();

        let flag_array = grid.iter()
            .map(|cell| cell.action == CellAction::Flagged)
            .collect();

        let turns = native_turns.as_ref().map(|native_turns| {
            native_turns.iter()
                .map(|turn| Turn::from_native(turn, server))
                .collect()
        });

        Game {
            id: None,
            created_at,
            pass: None,
            seed: seed as i32,
            dims: dims.iter().map(|&d| d as i32).collect(),
            size: dims.iter().fold(1, |acc, &d| acc * d) as i32,
            mines: mines as i32,
            autoclear,
            turns,
            clients: vec!["RustoBusto".to_owned()],
            cell_array,
            flag_array
        }
    }
}

impl Turn {
    fn from_native(turn_info: &TurnInfo, server: &NativeServer) -> Turn {
        let to_coords_vec = |indices: &[usize]| -> Vec<Coords<i32>> {
            indices.iter()
                .map(|&i| Coords::from_index(i, &server.dims))
                .collect()
        };

        let clear_actual = turn_info.clear_actual.iter()
            .map(|&i| {
                let &Cell {
                    mine,
                    surr_mine_count,
                    ..
                } = &server.grid[i];

                let state = if mine {
                    CellState::Mine
                } else {
                    CellState::Cleared
                };

                CellInfo {
                    surrounding: surr_mine_count as i32,
                    state,
                    coords: Coords::from_index(i, &server.dims)
                }
            })
            .collect();

        Turn {
            turn_taken_at: turn_info.timestamp.clone(),
            clear_req: to_coords_vec(&turn_info.clear_req),
            clear_actual,
            flagged: to_coords_vec(&turn_info.flagged),
            unflagged: to_coords_vec(&turn_info.unflagged),
            game_over: turn_info.game_state != GameState::Ongoing,
            win: turn_info.game_state == GameState::Win,
            cells_rem: turn_info.cells_rem as i32
        }
    }
}

pub fn insert_game(server: &NativeServer) -> mongodb::Result<()> {
    Game::from_native(server).save(DB_CONNECTION.clone(), None)
}