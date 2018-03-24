use itertools::Itertools;

use std::collections::HashSet;
use std::iter::repeat;
use std::convert::TryInto;
use cell::{Cell, Action, SingleCellAction};
use server_wrapper::server_response::{CellInfo, CellState};
use action_queue::ActionQueue;

pub type Coords = Vec<usize>;

#[derive(Debug)]
pub struct ServerActions {
    pub to_clear: Vec<Coords>,
    pub to_flag: Vec<Coords>
}

pub struct GameGrid {
    dims: Coords,
    cells: Vec<Cell>,
}

impl GameGrid {
    pub fn new(dims: Coords) -> Self {
        let all_coords = dims.iter().map(|&d| 0..d).multi_cartesian_product();

        let cells = all_coords.enumerate().map(|(i, coords)| {
            Cell::new(i, surr_indices(&coords, &dims))
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
            let index = coords_to_index(coords, &dims);
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
                .map(|&i| index_to_coords(i, &dims))
                .collect(),
            to_flag: actions.get_to_flag()
                .map(|&i| index_to_coords(i, &dims))
                .collect(),
        };
        let game_grid = GameGrid { dims, cells };

        (game_grid, next_actions)
    }
}

fn surr_indices(coords: &Coords, dims: &Coords) -> HashSet<usize> {
    let offsets = repeat(-1..=1).take(dims.len())
        .multi_cartesian_product()
        .filter(|offset| offset.iter().any(|&c| c != 0));

    let surr_coords = offsets.filter_map(|offset| -> Option<Coords> {
        let mut surr = vec![];

        for ((o, &c), &d) in offset.into_iter()
            .zip(coords.iter())
            .zip(dims.iter())
        {
            let s = (o + (c as isize)).try_into().ok()?;
            if s >= d { return None; }
            surr.push(s);
        }

        Some(surr)
    });

    surr_coords.map(|s| coords_to_index(&s, dims)).collect()
}

fn coords_to_index(coords: &Coords, dims: &Coords) -> usize {
    coords.iter().zip(dims.iter())
        .fold(0, |acc, (&coord, &dim)| (acc * dim) + coord)
}

fn index_to_coords(index: usize, dims: &Coords) -> Coords {
    let mut coords: Coords = dims.iter()
        .rev()
        .scan(index, |rem, &dim| {
            let coord = *rem % dim;
            *rem /= dim;
            Some(coord)
        })
        .collect();

    coords.reverse();

    coords
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

#[test]
fn test_surr_indices() {
    for (coords, dims, exp) in vec![
        (vec![5, 5], vec![10, 10], vec![44, 45, 46, 54, 56, 64, 65, 66]),
        (vec![5, 9], vec![10, 10], vec![48, 49, 58, 68, 69]),
        (vec![9, 5], vec![10, 10], vec![84, 85, 86, 94, 96]),
        (vec![9, 0], vec![10, 10], vec![80, 81, 91]),
        (vec![0, 0], vec![1, 1], vec![]),
        (vec![1, 0], vec![2, 2], vec![0, 1, 3])
    ] {
        let exp_set = exp.into_iter().collect();
        assert_eq!(surr_indices(&coords, &dims), exp_set);
    }
}