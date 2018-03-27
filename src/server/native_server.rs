extern crate rand;
extern crate chrono;

use std::fmt;
use std::error::Error;
use std::collections::{VecDeque, HashSet};
use self::rand::{Rng, thread_rng};
use self::chrono::{DateTime, Utc};

use coords::Coords;
use server::GameServer;
use server::json_api::resp::{ServerResponse, CellInfo, CellState};
use game_grid::GameGrid;

#[derive(Clone, Copy, Debug, PartialEq)]
enum CellAction { NoAction, Flagged, Cleared }

#[derive(Clone, Copy, Debug, PartialEq)]
enum GameState { Ongoing, Win, Lose }

struct Cell {
    mine: bool,
    action: CellAction,
    surr_indices: HashSet<usize>
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

pub struct NativeServer {
    created_at: DateTime<Utc>,
    grid: GameGrid<Cell>,
    mines: usize,
    autoclear: bool,
    cells_rem: usize,
    game_state: GameState,
    turns: Vec<ServerResponse>,
}

impl NativeServer {
    fn new(dims: Vec<usize>, mines: usize, autoclear: bool) -> Self {
        let size = dims.iter().fold(1, |s, &i| s * i);

        if dims.len() == 0 || mines >= size {
            panic!(
                "Invalid game params: dims={:?} mines={:?} autoclear={:?}",
                dims,
                mines,
                autoclear
            );
        }

        let mut grid = GameGrid::new(dims, |_i, surr| Cell {
            mine: false,
            action: CellAction::NoAction,
            surr_indices: surr
        });

        let mut rng = thread_rng();

        // Place mines randomly using Fisher-Yates shuffle
        for i in 0..grid.cells.len() {
            let rand = rng.gen_range(0, i);

            if rand != i {
                grid.cells[i].mine = grid.cells[rand].mine;
            }

            grid.cells[rand].mine = i <= mines;
        }

        let mut server = NativeServer {
            created_at: Utc::now(),
            mines,
            autoclear,
            grid,
            cells_rem: size - mines,
            game_state: GameState::Ongoing,
            turns: Vec::new()
        };
        let first_turn = server.turn_info(vec![], vec![], vec![], vec![]);

        server.turns.push(first_turn);

        server
    }

    fn clear_cells(&mut self, to_clear: &[Coords]) -> Vec<CellInfo> {
        let mut clear_actual = Vec::new();
        let mut coords_stack: VecDeque<(usize, Coords)> = to_clear.iter()
            .map(|coords| (coords.to_index(&self.grid.dims), coords.clone()))
            .collect();

        while let Some((index, coords)) = coords_stack.pop_front() {
            let cell = &mut self.grid.cells[index];

            if let Err(_) = cell.set_action(CellAction::Cleared) { continue; }

            clear_actual.push(CellInfo {
                surrounding: cell.surr_indices.len(),
                state: if cell.mine { CellState::Mine } else { CellState::Cleared },
                coords: coords
            });

            if cell.mine {
                self.game_state = GameState::Lose;
            } else {
                if self.autoclear && cell.surr_indices.len() == 0 {
                    for &i in cell.surr_indices.iter() {
                        coords_stack.push_front(
                            (i, Coords::from_index(i, &self.grid.dims))
                        );
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
    ) -> Vec<Coords> {
        let mut actual_change = Vec::new();

        for coords in to_change.into_iter() {
            let index = coords.to_index(&self.grid.dims);
            let cell = &mut self.grid.cells[index];

            if let Ok(_) = cell.set_action(new_action) {
                actual_change.push(coords);
            }
        }

        actual_change
    }

    fn turn_info(
        &self,
        clear_req: Vec<Coords>,
        clear_actual: Vec<CellInfo>,
        flagged: Vec<Coords>,
        unflagged: Vec<Coords>
    ) -> ServerResponse {
        ServerResponse {
            id: "someid".to_string(),
            seed: 0,
            dims: self.grid.dims.clone(),
            mines: self.mines,
            turn_num: self.turns.len(),
            game_over: self.game_state != GameState::Ongoing,
            win: self.game_state == GameState::Win,
            cells_rem: self.cells_rem,
            flagged,
            unflagged,
            clear_actual,
            clear_req,
            turn_taken_at: Utc::now()
        }
    }
}

impl GameServer for NativeServer {
    fn turn(
        &mut self,
        clear: Vec<Coords>,
        flag: Vec<Coords>,
        unflag: Vec<Coords>,
    ) -> Result<(), Box<Error>> {
        if self.game_state != GameState::Ongoing {
            return Err(GameError(String::from("Game already finished")))?;
        }

        let clear_actual = self.clear_cells(&clear);
        let flag_actual = self.set_flags(flag, CellAction::Flagged);
        let unflag_actual = self.set_flags(unflag, CellAction::NoAction);

        let turn_info = self.turn_info(
            clear,
            clear_actual,
            flag_actual,
            unflag_actual
        );

        self.turns.push(turn_info);

        Ok(())
    }

    fn status(&self) -> &ServerResponse {
        self.turns.last().unwrap()
    }
}

#[derive(Debug)]
struct GameError(String);

impl Error for GameError {
    fn description(&self) -> &str { &self.0 }
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}