extern crate tokio_core;

use std::error::Error;
use coords::Coords;
use client::cell::{Cell, Action, SingleCellAction};
use client::action_queue::ActionQueue;
use server::json_api::resp::{CellInfo, CellState};
use server::json_server_wrapper::JsonServerWrapper;
use server::GameServer;
use game_grid::GameGrid;
use util::index_pair;
use self::tokio_core::reactor;

#[derive(Debug)]
struct ServerActions {
    to_clear: Vec<Coords>,
    to_flag: Vec<Coords>
}

pub struct Client<G: GameServer> {
    grid: GameGrid,
    server: G
}

impl<'a> Client<JsonServerWrapper<'a>> {
    pub fn new(
        dims: Vec<usize>,
        mines: usize,
        seed: Option<u64>,
        autoclear: bool,
        event_loop_core: &'a mut reactor::Core
    ) -> Result<Self, Box<Error>> {
        let server = JsonServerWrapper::new_game(
            dims.clone(),
            mines,
            seed,
            autoclear,
            event_loop_core,
        )?;
        let grid = GameGrid::new(dims);

        Ok(Client { grid, server })
    }

    pub fn play(&mut self) -> Result<bool, Box<Error>> {
        let mut to_clear = vec![Coords(
            self.grid.dims.iter().map(|&d| d / 2).collect()
        )];
        let mut to_flag = vec![];

        println!("{:?}", self.server.status());

        while !(to_clear.is_empty() && to_flag.is_empty()) {
            println!("Turn {}", self.server.status().turn_num);
            println!("to_clear {:?}", to_clear);
            println!("to_flag {:?}", to_flag);

            self.server.turn(to_clear, to_flag, vec![])?;

            if self.server.status().game_over { break; }

            let next_actions = self.next_turn();

            to_clear = next_actions.to_clear;
            to_flag = next_actions.to_flag;
        }

        Ok(self.server.status().win)
    }

    fn next_turn(&mut self) -> ServerActions {
        let mut actions = ActionQueue::new();

        for &CellInfo {
            surrounding,
            state,
            ref coords
        } in self.server.status().clear_actual.iter() {
            let index = coords.to_index(&self.grid.dims);
            let action_type = match state {
                CellState::Cleared => SingleCellAction::ClientClear {
                    mines: surrounding
                },
                CellState::Mine => SingleCellAction::Flag
            };

            actions.push(Action::Single { index, action_type });
        }

        while let Some(action) = actions.pull() {
            match action {
                Action::Single { index, action_type } => {
                    let cell = &mut self.grid.cells[index];
                    let mut complete = false;

                    if let &mut Cell::Ongoing(ref mut ongoing) = cell {
                        complete = ongoing.apply_action(&mut actions, action_type);
                    }

                    if complete {
                        *cell = Cell::Complete;
                    }
                },
                Action::Pair { index1, index2, action_type } => {
                    match index_pair(&mut self.grid.cells, index1, index2) {
                        (
                            &mut Cell::Ongoing(ref mut cell1),
                            &mut Cell::Ongoing(ref mut cell2)
                        ) => {
                            cell1.apply_pair_action(
                                cell2,
                                &mut actions,
                                action_type
                            );
                        },
                        _ => ()
                    }
                }
            }
        }

        let next_actions = ServerActions {
            to_clear: actions.get_to_clear()
                .map(|&i| Coords::from_index(i, &self.grid.dims))
                .collect(),
            to_flag: actions.get_to_flag()
                .map(|&i| Coords::from_index(i, &self.grid.dims))
                .collect(),
        };

        next_actions
    }
}