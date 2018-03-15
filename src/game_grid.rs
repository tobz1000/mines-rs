use itertools::Itertools;

use cell::{Cell, ClientAction, ServerAction};
use server_wrapper::server_response::{CellInfo, CellState};
use std::collections::{VecDeque, HashSet};

pub struct ServerActions {
    pub to_clear: Vec<Vec<usize>>,
    pub to_flag: Vec<Vec<usize>>
}

pub struct GameGrid {
	dims: Vec<usize>,
    offsets: Vec<usize>,
	arr: Vec<Option<Cell>>
}

impl GameGrid {
	pub fn new(dims: Vec<usize>) -> Self {
		let arr_size = dims.iter().fold(1, |a, b| a * b) as usize;
		let offsets = surr_offsets(dims.as_slice());

		let arr = vec![None; arr_size];

		GameGrid { dims, offsets, arr }
	}

    fn get_cell(&mut self, index: usize) -> &mut Cell {
        let surr_count = self.surr_indices(index).len();
        let cell = &mut self.arr[index];

        if let &mut None = cell {
            *cell = Some(Cell::new(surr_count));
        }

        cell.as_mut().unwrap()
    }

    fn surr_indices(&mut self, index: usize) -> Vec<usize> {
        self.offsets.iter()
            .filter_map(|&offset| {
                let surr_index = index as isize + offset as isize;
                if (0..self.arr.len() as isize).contains(surr_index) {
                    Some(surr_index as usize)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn handle_cell_info(&mut self, cell_info: &[CellInfo]) -> ServerActions {
        let mut client_actions = VecDeque::new();
        let mut server_to_clear = HashSet::new();
        let mut server_to_flag = HashSet::new();

        for &CellInfo {
            surrounding,
            state,
            ref coords
        } in cell_info.iter() {
            let index = self.coords_to_index(coords);
            let action = match state {
                CellState::cleared => ClientAction::Clear(surrounding),
                CellState::mine => ClientAction::Flag
            };
            client_actions.push_back((index, action));
        }

        while let Some((index, action)) = client_actions.pop_front() {
            let surr_indices = self.surr_indices(index);
            let cell = self.get_cell(index);

            if let Some(server_action) = match action {
                ClientAction::Clear(surr_mine_count) => {
                    for &surr in surr_indices.iter() {
                        client_actions.push_back((surr, ClientAction::IncSurrEmpty));
                    }

                    cell.set_clear(surr_mine_count)
                },
                ClientAction::Flag => {
                    for &surr in surr_indices.iter() {
                        client_actions.push_back((surr, ClientAction::IncSurrMine));
                    }

                    cell.set_mine();
                    None
                },
                ClientAction::IncSurrEmpty => cell.inc_surr_empty(),
                ClientAction::IncSurrMine => cell.inc_surr_mine(),
            } {
                for &surr in surr_indices.iter() {
                    match server_action {
                        ServerAction::Clear => { server_to_clear.insert(surr); },
                        ServerAction::Flag => { server_to_flag.insert(surr); },
                    }
                }
            }
        }

        ServerActions {
            to_clear: server_to_clear.into_iter()
                .map(|i| self.index_to_coords(i))
                .collect(),
            to_flag: server_to_flag.into_iter()
                .map(|i| self.index_to_coords(i))
                .collect(),
        }
    }

    fn coords_to_index(&self, coords: &[usize]) -> usize {
        coords.iter().zip(self.dims.iter())
            .fold(0, |acc, (&coord, &dim)| {
                (acc * dim) + coord
            }) as usize
    }

    fn index_to_coords(&self, index: usize) -> Vec<usize> {
        let mut coords: Vec<usize> = self.dims.iter()
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
}

fn surr_offsets(dims: &[usize]) -> Vec<usize>{
    dims.iter()
        .scan(0, |state, &dim| {
            *state *= dim;
            Some(*state)
        })
        .map(|acc_dim| vec![
            -(acc_dim as isize),
            0,
            acc_dim as isize
        ].into_iter())
        .multi_cartesian_product()
        .map(|offsets| offsets.into_iter().fold(0, |acc, o| acc + o) as usize)
        .filter(|&offset| offset == 0)
        .collect()
}

// fn coords_to_index(dims: &[i32], coords: &[i32]) -> usize {
//     coords.iter().zip(dims.iter())
//         .fold(0, |acc, (&coord, &dim)| {
//             (acc * dim) + coord
//         }) as usize
// }

// fn index_to_coords(dims: &[i32], index: usize) -> Vec<i32> {
//     dims.iter().rev().scan(index, |rem, &dim| {
//         let coord = *rem % dim;
//         *rem /= dim;
//         coord
//     }).rev().collect()
// }