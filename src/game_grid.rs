use itertools::Itertools;

use coords::Coords;
use cell::{Cell, Action, SingleCellAction};
use server::json_server_wrapper::server_response::{CellInfo, CellState};
use action_queue::ActionQueue;

#[derive(Debug)]
pub struct ServerActions {
    pub to_clear: Vec<Coords>,
    pub to_flag: Vec<Coords>
}

pub struct GameGrid {
    dims: Vec<usize>,
    cells: Vec<Cell>,
}

impl GameGrid {
    pub fn new(dims: Vec<usize>) -> Self {
        let all_coords = dims.iter().map(|&d| 0..d).multi_cartesian_product();

        let cells = all_coords.enumerate().map(|(i, coords)| {
            Cell::new(i, Coords(coords).surr_indices(&dims))
        }).collect();

        GameGrid { dims, cells }
    }

    pub fn next_turn(self, cell_info: &[CellInfo]) -> (Self, ServerActions) {
        let mut actions = ActionQueue::new();

        let GameGrid { dims, mut cells } = self;

        for &CellInfo {
            surrounding,
            state,
            ref coords
        } in cell_info.iter() {
            let index = coords.to_index(&dims);
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
                    let cell = &mut cells[index];
                    let mut complete = false;

                    if let &mut Cell::Ongoing(ref mut ongoing) = cell {
                        complete = ongoing.apply_action(&mut actions, action_type);
                    }

                    if complete {
                        *cell = Cell::Complete;
                    }
                },
                Action::Pair { index1, index2, action_type } => {
                    match index_pair(&mut cells, index1, index2) {
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
                .map(|&i| Coords::from_index(i, &dims))
                .collect(),
            to_flag: actions.get_to_flag()
                .map(|&i| Coords::from_index(i, &dims))
                .collect(),
        };
        let game_grid = GameGrid { dims, cells };

        (game_grid, next_actions)
    }
}

fn index_pair<T>(slice: &mut [T], ia: usize, ib: usize) -> (&mut T, &mut T) {
    if ia == ib || ia > slice.len() || ib > slice.len() {
        panic!(
            "Invalid index pair ({}, {}); slice.len() == {}",
            ia,
            ib,
            slice.len()
        );
    }

    let a;
    let b;

    unsafe {
        a = &mut *(slice.get_unchecked_mut(ia) as *mut _);
        b = &mut *(slice.get_unchecked_mut(ib) as *mut _);
    }

    (a, b)
}
