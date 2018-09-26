extern crate rand;
#[cfg(feature = "chrono")] extern crate chrono;
extern crate itertools;
extern crate mersenne_twister;

use std::collections::HashSet;
use ::GameError;
use self::rand::{Rng, SeedableRng};
#[cfg(feature = "chrono")] use self::chrono::{DateTime, Utc};
use self::itertools::Itertools;
use self::mersenne_twister::MT19937;

use coords::Coords;
use server::{GameServer, GameState, GameSpec, CellInfo};
use game_grid::GameGrid;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CellAction { NoAction, Flagged, Cleared }

#[derive(Debug)]
pub struct Cell {
    pub mine: bool,
    pub action: CellAction,
    pub surr_indices: HashSet<usize>,
    pub surr_mine_count: usize
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
    #[cfg(feature = "chrono")] pub created_at: DateTime<Utc>,
    pub dims: Vec<usize>,
    pub grid: GameGrid<Cell>,
    pub mines: usize,
    pub seed: u32,
    pub autoclear: bool,
    pub turns: Option<Vec<TurnInfo>>,
    pub cells_rem: usize,
    pub game_state: GameState,
}

#[derive(Debug)]
pub struct TurnInfo {
    #[cfg(feature = "chrono")] pub timestamp: DateTime<Utc>,
    pub clear_req: Vec<usize>,
    pub clear_actual: Vec<usize>,
    pub flagged: Vec<usize>,
    pub unflagged: Vec<usize>,
    pub cells_rem: usize,
    pub game_state: GameState,
}

impl NativeServer {
    pub fn new(
        GameSpec {
            dims,
            mines,
            seed,
            autoclear
        }: GameSpec,
        store_turns: bool
    ) -> Result<Self, GameError> {
        let size = dims.iter().fold(1, |s, &i| s * i);
        let cells_rem = size - mines;
        let game_state = GameState::Ongoing;

        if dims.len() == 0 || mines >= size {
            panic!(
                "Invalid game params: dims={:?} mines={:?} autoclear={:?}",
                dims,
                mines,
                autoclear
            );
        }

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

        let turns = if store_turns {
            Some(vec![
                TurnInfo {
                    #[cfg(feature = "chrono")] timestamp: Utc::now(),
                    clear_req: Vec::new(),
                    clear_actual: Vec::new(),
                    flagged: Vec::new(),
                    unflagged: Vec::new(),
                    cells_rem,
                    game_state,
                }
            ])
        } else {
            None
        };

        Ok(NativeServer {
            #[cfg(feature = "chrono")] created_at: Utc::now(),
            dims,
            mines,
            seed,
            autoclear,
            grid,
            cells_rem,
            game_state,
            turns
        })
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

        if let Some(ref mut turns) = self.turns {
            let turn_info = TurnInfo {
                #[cfg(feature = "chrono")] timestamp: Utc::now(),
                clear_req: clear_req_indices,
                clear_actual: clear_actual.clone(),
                flagged: flag_actual,
                unflagged: unflag_actual,
                cells_rem: self.cells_rem,
                game_state: self.game_state,
            };

            turns.push(turn_info);
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