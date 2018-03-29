extern crate rand;
extern crate chrono;
extern crate itertools;

use std::fmt;
use std::error::Error;
use std::collections::{VecDeque, HashSet};
use self::rand::{Rng, thread_rng};
use self::chrono::{DateTime, Utc};
use self::itertools::Itertools;

use coords::Coords;
use server::GameServer;
use server::json_api::resp::{ServerResponse, CellInfo, CellState};
use game_grid::GameGrid;

#[derive(Debug)]
pub struct GameError(String);

impl Error for GameError {
    fn description(&self) -> &str { &self.0 }
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum CellAction { NoAction, Flagged, Cleared }

#[derive(Clone, Copy, Debug, PartialEq)]
enum GameState { Ongoing, Win, Lose }

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

pub struct NativeServer {
    created_at: DateTime<Utc>,
    dims: Vec<usize>,
    grid: GameGrid<Cell>,
    mines: usize,
    autoclear: bool,
    cells_rem: usize,
    game_state: GameState,
    turns: Vec<ServerResponse>,
}

impl NativeServer {
    pub fn new(dims: Vec<usize>, mines: usize, autoclear: bool) -> Self {
        let size = dims.iter().fold(1, |s, &i| s * i);

        if dims.len() == 0 || mines >= size {
            panic!(
                "Invalid game params: dims={:?} mines={:?} autoclear={:?}",
                dims,
                mines,
                autoclear
            );
        }

        let grid: GameGrid<Cell> = {
            let mut rng = thread_rng();

            // Place mines randomly using Fisher-Yates shuffle
            let mut mine_arr = vec![false; size];

            for i in 0..size {
                let rand = if i == 0 { 0 } else { rng.gen_range(0, i) };

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

    pub fn grid_repr(&self) -> Result<String, GameError> {
        if self.dims.len() > 2 {
            return Err(GameError(format!(
                "Can only repr game of <= 2 dimensions; dims={:?}",
                self.dims
            )));
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

    fn clear_cells(&mut self, to_clear: &[Coords]) -> Vec<CellInfo> {
        let mut clear_actual = Vec::new();
        let mut coords_stack: VecDeque<(usize, Coords)> = to_clear.iter()
            .map(|coords| (coords.to_index(&self.dims), coords.clone()))
            .collect();

        while let Some((index, coords)) = coords_stack.pop_front() {
            let cell = &mut self.grid[index];

            if let Err(_) = cell.set_action(CellAction::Cleared) { continue; }

            clear_actual.push(CellInfo {
                surrounding: cell.surr_mine_count,
                state: if cell.mine { CellState::Mine } else { CellState::Cleared },
                coords: coords
            });

            if cell.mine {
                self.game_state = GameState::Lose;
            } else {
                if self.autoclear && cell.surr_mine_count == 0 {
                    for &i in cell.surr_indices.iter() {
                        coords_stack.push_front(
                            (i, Coords::from_index(i, &self.dims))
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
            let index = coords.to_index(&self.dims);
            let cell = &mut self.grid[index];

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
            dims: self.dims.clone(),
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

        if let Ok(repr) = self.grid_repr() {
            println!("{}", repr);
        }

        Ok(())
    }

    fn status(&self) -> &ServerResponse {
        self.turns.last().unwrap()
    }
}