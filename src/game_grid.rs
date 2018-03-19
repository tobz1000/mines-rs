use itertools::Itertools;

use std::collections::{VecDeque, HashSet};
use std::iter::repeat;
use std::convert::TryInto;
use cell::{Cell, CellAction};
use server_wrapper::server_response::{CellInfo, CellState};

pub type Coords = Vec<usize>;

#[derive(Debug)]
pub struct ServerActions {
    pub to_clear: Vec<Coords>,
    pub to_flag: Vec<Coords>
}

pub struct GameGrid {
    dims: Coords,
    cells: Vec<(Cell, Vec<usize>)>,
}

impl GameGrid {
    pub fn new(dims: Coords) -> Self {
        let all_coords = dims.iter().map(|&d| 0..d).multi_cartesian_product();

        let cells = all_coords.map(|coords| {
            let surr_indices = surr_indices(&coords, &dims);
            (Cell::new(surr_indices.len()), surr_indices)
        }).collect();

        GameGrid { dims, cells }
    }

    pub fn next_turn(self, cell_info: &[CellInfo]) -> (Self, ServerActions) {
        let mut actions = VecDeque::new();
        let mut server_to_clear = HashSet::new();
        let mut server_to_flag = HashSet::new();

        let GameGrid { dims, mut cells } = self;

        for &CellInfo {
            surrounding,
            state,
            ref coords
        } in cell_info.iter() {
            let index = coords_to_index(coords, &dims);
            let action = match state {
                CellState::Cleared => CellAction::ClientClear(surrounding),
                CellState::Mine => CellAction::Flag
            };
            actions.push_back((index, action));
        }

        while let Some((index, client_action)) = actions.pop_front() {
            let (ref mut cell, ref surr_indices) = cells[index];

            if let Some(surr_action) = match client_action {
                CellAction::IncSurrEmpty => cell.inc_surr_empty(),
                CellAction::IncSurrMine => cell.inc_surr_mine(),
                CellAction::ClientClear(surr_mine_count) => {
                    for &surr in surr_indices.iter() {
                        actions.push_back(
                            (surr, CellAction::IncSurrEmpty)
                        );
                    }

                    cell.set_clear(surr_mine_count)
                },
                CellAction::ServerClear => {
                    if cell.to_submit() {
                        server_to_clear.insert(index);
                    }

                    None
                }
                CellAction::Flag => {
                    if cell.to_submit() {
                        server_to_flag.insert(index);

                        for &surr in surr_indices.iter() {
                            actions.push_back(
                                (surr, CellAction::IncSurrMine)
                            );
                        }

                        cell.set_mine();
                    }

                    None
                },
            } {
                for &surr_index in surr_indices.iter() {
                    actions.push_back((surr_index, surr_action));
                }
            }
        }

        let next_actions = ServerActions {
            to_clear: server_to_clear.into_iter()
                .map(|i| index_to_coords(i, &dims))
                .collect(),
            to_flag: server_to_flag.into_iter()
                .map(|i| index_to_coords(i, &dims))
                .collect(),
        };
        let game_grid = GameGrid { dims, cells };

        (game_grid, next_actions)
    }
}

fn surr_indices(coords: &Coords, dims: &Coords) -> Vec<usize> {
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

#[test]
fn test_surr_indices() {
    for (coords, dims, exp) in vec![
        (vec![5, 5], vec![10, 10], vec![44, 45, 46, 54, 56, 64, 65, 66]),
        (vec![5, 9], vec![10, 10], vec![48, 49, 58, 68, 69]),
        (vec![9, 5], vec![10, 10], vec![84, 85, 86, 94, 96]),
        (vec![9, 0], vec![10, 10], vec![80, 81, 91]),
    ] {
        assert_eq!(surr_indices(&coords, &dims), exp);
    }
}