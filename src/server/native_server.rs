extern crate rand;
extern crate chrono;
extern crate itertools;
extern crate mersenne_twister;
extern crate mongodb;
extern crate wither;
use std::collections::HashSet;
use ::GameError;
use self::rand::{Rng, thread_rng, SeedableRng};
use self::chrono::{DateTime, Utc};
use self::itertools::Itertools;
use self::mersenne_twister::MT19937;
use self::mongodb::db::Database;
use self::wither::Model;

use coords::Coords;
use server::{GameServer, GameState, CellInfo};
use server::db;
use game_grid::GameGrid;

#[derive(Clone, Copy, Debug, PartialEq)]
enum CellAction { NoAction, Flagged, Cleared }

#[derive(Debug)]
struct Cell {
    mine: bool,
    action: CellAction,
    surr_indices: HashSet<usize>,
    surr_mine_count: usize
}

impl Cell {
    fn set_action(&mut self, new_action: CellAction) -> Result<(), ()> {
        if self.action == new_action || self.action == CellAction::Cleared {
            Err(())
        } else {
            self.action = new_action;
            Ok(())
        }
    }
}

struct TurnInfo {
    timestamp: DateTime<Utc>,
    clear_req: Vec<usize>,
    clear_actual: Vec<usize>,
    flagged: Vec<usize>,
    unflagged: Vec<usize>,
    cells_rem: usize,
    game_state: GameState,
}

impl TurnInfo {
    fn to_document(&self, server: &NativeServer) -> db::Turn {
        let to_coords_vec = |indices: &[usize]| -> Vec<Coords<i32>> {
            indices.iter()
                .map(|&i| Coords::from_index(i, &server.dims))
                .collect()
        };

        let clear_actual = self.clear_actual.iter()
            .map(|&i| {
                let &Cell {
                    mine,
                    surr_mine_count,
                    ..
                } = &server.grid[i];

                let state = if mine {
                    db::CellState::Mine
                } else {
                    db::CellState::Cleared
                };

                db::CellInfo {
                    surrounding: surr_mine_count,
                    state,
                    coords: Coords::from_index(i, &server.dims)
                }
            })
            .collect();

        db::Turn {
            turn_taken_at: self.timestamp.clone(),
            clear_req: to_coords_vec(&self.clear_req),
            clear_actual,
            flagged: to_coords_vec(&self.flagged),
            unflagged: to_coords_vec(&self.unflagged),
            game_over: self.game_state != GameState::Ongoing,
            win: self.game_state == GameState::Win,
            cells_rem: self.cells_rem as i32
        }
    }
}

pub struct NativeServer {
    created_at: DateTime<Utc>,
    dims: Vec<usize>,
    grid: GameGrid<Cell>,
    mines: usize,
    seed: u32,
    autoclear: bool,
    cells_rem: usize,
    game_state: GameState,
    turns: Vec<TurnInfo>,
    db_connection: Option<Database>,
}

impl NativeServer {
    pub fn new(
        dims: Vec<usize>,
        mines: usize,
        user_seed: Option<u32>,
        autoclear: bool,
        db_connection: Option<Database>
    ) -> Self {
        let size = dims.iter().fold(1, |s, &i| s * i);

        if dims.len() == 0 || mines >= size {
            panic!(
                "Invalid game params: dims={:?} mines={:?} autoclear={:?}",
                dims,
                mines,
                autoclear
            );
        }

        let seed = if let Some(seed) = user_seed {
            seed
        } else {
            thread_rng().gen()
        };

        let grid: GameGrid<Cell> = {
            let mut rng: MT19937 = SeedableRng::from_seed(seed);

            // Place mines randomly using Fisher-Yates shuffle
            let mut mine_arr = vec![false; size];

            for i in 0..size {
                // Avoid rng-generation on first iteration to match the JS
                // server's behaviour
                let rand = if i == 0 {
                    0
                } else {
                    rng.gen_range(0, (i + 1) as i32) as usize
                };

                if rand != i {
                    mine_arr[i] = mine_arr[rand];
                }

                mine_arr[rand] = i < mines;
            }

            GameGrid::new(&dims, |i, surr| {
                let surr_mine_count = surr.iter()
                    .filter(|&&s| mine_arr[s])
                    .count();

                Cell {
                    mine: mine_arr[i],
                    action: CellAction::NoAction,
                    surr_indices: surr,
                    surr_mine_count
                }
            })
        };

        let mut server = NativeServer {
            created_at: Utc::now(),
            dims,
            mines,
            seed,
            autoclear,
            grid,
            cells_rem: size - mines,
            game_state: GameState::Ongoing,
            turns: Vec::new(),
            db_connection
        };

        if let Some(_) = server.db_connection {
            let first_turn = server.turn_info(vec![], vec![], vec![], vec![]);

            server.turns.push(first_turn);
        }

        server
    }

    #[allow(dead_code)]
    pub fn grid_repr(&self) -> Result<String, String> {
        if self.dims.len() > 2 {
            return Err(format!(
                "Can only repr game of <= 2 dimensions; dims={:?}",
                self.dims
            ));
        }

        let cell_repr = |x, y| {
            let index = Coords(vec![x, y]).to_index(&self.dims);
            let cell = &self.grid[index];
            match cell.action {
                CellAction::NoAction => '□',
                CellAction::Flagged => '⚐',
                CellAction::Cleared => {
                    if cell.mine {
                        '☢'
                    } else {
                        match cell.surr_mine_count {
                            0 => ' ',
                            c => (c as u8 + '0' as u8) as char
                        }
                    }
                }
            }
        };

        let row_repr = |y| (0..self.dims[0])
            .map(|x| cell_repr(x, y))
            .join(" ");

        let row_count = *self.dims.get(1).unwrap_or(&1);

        Ok((0..row_count).map(row_repr).join("\n"))
    }

    fn to_document(&self) -> db::Game {
        let &NativeServer {
            created_at,
            ref dims,
            ref grid,
            mines,
            seed,
            autoclear,
            ref turns,
            ..
        } = self;

        let cell_array = grid.iter().map(|&Cell { mine, action, .. }| {
            match (mine, action) {
                (true, _) => db::CellState::Mine,
                (false, CellAction::Cleared) => db::CellState::Cleared,
                (false, _) => db::CellState::Empty
            }
        }).collect();

        let flag_array = grid.iter()
            .map(|cell| cell.action == CellAction::Flagged)
            .collect();

        let turns = turns.iter().map(|turn| turn.to_document(self)).collect();

        db::Game {
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

    fn clear_cells(&mut self, mut to_clear: Vec<usize>) -> Vec<usize> {
        let mut clear_actual = Vec::new();

        while let Some(index) = to_clear.pop() {
            let cell = &mut self.grid[index];

            if let Err(_) = cell.set_action(CellAction::Cleared) { continue; }

            clear_actual.push(index);

            if cell.mine {
                self.game_state = GameState::Lose;
            } else {
                if self.autoclear && cell.surr_mine_count == 0 {
                    for &i in cell.surr_indices.iter() {
                        to_clear.push(i);
                    }
                }

                self.cells_rem -= 1;

                if self.cells_rem == 0 {
                    self.game_state = GameState::Win;
                }
            }
        }

        clear_actual
    }

    fn set_flags(
        &mut self,
        to_change: Vec<Coords>,
        new_action: CellAction
    ) -> Vec<usize> {
        let mut actual_change = Vec::new();

        for coords in to_change.into_iter() {
            let index = coords.to_index(&self.dims);
            let cell = &mut self.grid[index];

            if let Ok(_) = cell.set_action(new_action) {
                actual_change.push(index);
            }
        }

        actual_change
    }

    fn turn_info(
        &self,
        clear_req: Vec<usize>,
        clear_actual: Vec<usize>,
        flagged: Vec<usize>,
        unflagged: Vec<usize>
    ) -> TurnInfo {
        TurnInfo {
            timestamp: Utc::now(),
            clear_req,
            clear_actual,
            flagged,
            unflagged,
            cells_rem: self.cells_rem,
            game_state: self.game_state,
        }
    }

    fn client_cell_info(&self, index: usize) -> CellInfo {
        let cell = &self.grid[index];

        CellInfo {
            coords: Coords::from_index(index, &self.dims),
            mine: cell.mine,
            surrounding: cell.surr_mine_count
        }
    }
}

impl GameServer for NativeServer {
    fn turn(
        &mut self,
        clear: Vec<Coords>,
        flag: Vec<Coords>,
        unflag: Vec<Coords>,
    ) -> Result<Vec<CellInfo>, GameError> {
        if self.game_state != GameState::Ongoing {
            return Err(String::from("Game already finished"))?;
        }

        let clear_req_indices: Vec<usize> = clear.iter()
            .map(|coords| coords.to_index(&self.dims))
            .collect();

        let clear_actual = self.clear_cells(clear_req_indices.clone());
        let flag_actual = self.set_flags(flag, CellAction::Flagged);
        let unflag_actual = self.set_flags(unflag, CellAction::NoAction);

        if let Some(ref db_connection) = self.db_connection {
            let turn_info = self.turn_info(
                clear_req_indices,
                clear_actual.clone(),
                flag_actual,
                unflag_actual
            );

            self.turns.push(turn_info);

            // Intended to only save once - doc model is discarded, so _id is
            // not persisted.
            if self.game_state != GameState::Ongoing {
                self.to_document().save(db_connection.clone(), None)?;
            }
        }

        let client_cell_info = clear_actual.iter()
            .map(|&index| self.client_cell_info(index))
            .collect();

        Ok(client_cell_info)
    }

	fn dims(&self) -> &[usize] { &self.dims }

	fn mines(&self) -> usize { self.mines }

	fn game_state(&self) -> GameState { self.game_state }

	fn cells_rem(&self) -> usize { self.cells_rem }
}