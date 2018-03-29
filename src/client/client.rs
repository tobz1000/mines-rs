extern crate tokio_core;

use std::error::Error;
use coords::Coords;
use client::cell::{Cell, Action, SingleCellAction};
use client::action_queue::ActionQueue;
use server::json_api::resp::{CellInfo, CellState};
use server::GameServer;
use game_grid::GameGrid;

#[derive(Debug)]
struct ServerActions {
    to_clear: Vec<Coords>,
    to_flag: Vec<Coords>
}

pub struct Client<G: GameServer> {
    dims: Vec<usize>,
    grid: GameGrid<Cell>,
    server: G
}

impl<G: GameServer> Client<G> {
    pub fn new(server: G) -> Self {
        let dims = server.status().dims.clone();
        let grid = GameGrid::new(&dims, Cell::new);

        Client { dims, grid, server }
    }

    pub fn play(&mut self) -> Result<bool, Box<Error>> {
        let mut to_clear = vec![Coords(
            self.dims.iter().map(|&d| d / 2).collect()
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
            let index = coords.to_index(&self.dims);
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
                    let cell = &mut self.grid[index];
                    let mut complete = false;

                    if let &mut Cell::Ongoing(ref mut ongoing) = cell {
                        complete = ongoing.apply_action(&mut actions, action_type);
                    }

                    if complete {
                        *cell = Cell::Complete;
                    }
                },
                Action::Pair { index1, index2, action_type } => {
                    match self.grid.cell_pair(index1, index2) {
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
                .map(|&i| Coords::from_index(i, &self.dims))
                .collect(),
            to_flag: actions.get_to_flag()
                .map(|&i| Coords::from_index(i, &self.dims))
                .collect(),
        };

        next_actions
    }
}