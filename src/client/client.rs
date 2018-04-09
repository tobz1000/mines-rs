extern crate tokio_core;

use std::error::Error;
use coords::Coords;
use client::cell::{Cell, Action, SingleCellAction};
use client::action_queue::ActionQueue;
use server::{GameServer, GameState, CellInfo};
use game_grid::GameGrid;

#[derive(Debug)]
struct ServerActions {
    to_clear: Vec<Coords>,
    to_flag: Vec<Coords>
}

pub struct Client<G: GameServer> {
    grid: GameGrid<Cell>,
    server: G
}

impl<G: GameServer> Client<G> {
    pub fn new(server: G) -> Self {
        let grid = GameGrid::new(server.dims(), Cell::new);

        Client { grid, server }
    }

    pub fn play(&mut self) -> Result<GameState, Box<Error>> {
        let mut to_clear = vec![Coords(
            self.server.dims().iter().map(|&d| d / 2).collect()
        )];
        let mut to_flag = vec![];

        while !(to_clear.is_empty() && to_flag.is_empty()) {
            let clear_actual = self.server.turn(to_clear, to_flag, vec![])?;

            if self.server.game_state() != GameState::Ongoing { break; }

            let next_actions = self.next_turn(&clear_actual);

            to_clear = next_actions.to_clear;
            to_flag = next_actions.to_flag;
        }

        Ok(self.server.game_state())
    }

    fn next_turn(&mut self, clear_actual: &[CellInfo]) -> ServerActions {
        let mut actions = ActionQueue::new();

        for &CellInfo {
            ref coords,
            surrounding,
            mine,
        } in clear_actual.iter() {
            let index = coords.to_index(&self.server.dims());
            let action_type = if mine {
                SingleCellAction::Flag
            } else {
                SingleCellAction::ClientClear { mines: surrounding }
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

        if actions.get_to_clear().next() == None {
            actions.add_to_clear(self.guess_index());
        }

        let next_actions = ServerActions {
            to_clear: actions.get_to_clear()
                .map(|&i| Coords::from_index(i, &self.server.dims()))
                .collect(),
            to_flag: actions.get_to_flag()
                .map(|&i| Coords::from_index(i, &self.server.dims()))
                .collect(),
        };

        next_actions
    }

    fn guess_index(&self) -> usize {
        let (i, _cell) = self.grid.iter()
            .enumerate()
            .find(|&(_i, cell)| !cell.is_marked())
            .expect("Found no uncleared, unflagged cell to guess");

        i
    }
}