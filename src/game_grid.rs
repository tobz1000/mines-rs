use itertools::Itertools;

use cell::{Cell, CellAction};
use server_wrapper::server_response::{CellInfo, CellState};
use std::collections::{VecDeque, HashSet};

#[derive(Debug)]
pub struct ServerActions {
    pub to_clear: Vec<Vec<usize>>,
    pub to_flag: Vec<Vec<usize>>
}

pub struct GameGrid {
	dims: Vec<usize>,
    offsets: HashSet<isize>,
	cells: Vec<Option<Cell>>,
}

impl GameGrid {
	pub fn new(dims: Vec<usize>) -> Self {
		let arr_size = dims.iter().fold(1, |a, b| a * b) as usize;
		let offsets = surr_offsets(dims.as_slice());
		let cells = vec![None; arr_size];

		GameGrid { dims, offsets, cells }
	}

    pub fn next_turn(self, cell_info: &[CellInfo]) -> (Self, ServerActions) {
        let mut client_actions = VecDeque::new();
        let mut server_to_clear = HashSet::new();
        let mut server_to_flag = HashSet::new();

        let GameGrid { dims, offsets, mut cells } = self;

        for &CellInfo {
            surrounding,
            state,
            ref coords
        } in cell_info.iter() {
            let index = coords_to_index(coords, dims.as_slice());
            let action = match state {
                CellState::Cleared => CellAction::ClientClear(surrounding),
                CellState::Mine => CellAction::Flag
            };
            client_actions.push_back((index, action));
        }

        while let Some((index, client_action)) = client_actions.pop_front() {
            let surr_indices = surr_indices(&cells, &offsets, index);
            let cell = get_cell(&mut cells, &offsets, index);

            if index == 150 {
                println!("(10, 0) {:?}", client_action);
            }
            if let Some(surr_action) = match client_action {
                CellAction::IncSurrEmpty => cell.inc_surr_empty(),
                CellAction::IncSurrMine => cell.inc_surr_mine(),
                CellAction::ClientClear(surr_mine_count) => {
                    for &surr in surr_indices.iter() {
                        if surr == 150 {
                            println!(
                                "push (10, 0) {:?} from {:?}",
                                CellAction::IncSurrEmpty,
                                index_to_coords(index, &[15, 15])
                            );
                        }
                        client_actions.push_back(
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
                            client_actions.push_back(
                                (surr, CellAction::IncSurrMine)
                            );
                        }

                        cell.set_mine();
                    }

                    None
                },
            } {
                if index == 150 {
                    println!("(10, 0) {:?} (surr)", surr_action);
                }

                for &surr_index in surr_indices.iter() {
                    client_actions.push_back((surr_index, surr_action));
                }
            }
        }

        let next_actions = ServerActions {
            to_clear: server_to_clear.into_iter()
                .map(|i| index_to_coords(i, dims.as_slice()))
                .collect(),
            to_flag: server_to_flag.into_iter()
                .map(|i| index_to_coords(i, dims.as_slice()))
                .collect(),
        };
        let game_grid = GameGrid { dims, offsets, cells };

        (game_grid, next_actions)
    }
}

fn get_cell<'a>(
    cells: &'a mut[Option<Cell>], 
    offsets: &HashSet<isize>,
    index: usize
) -> &'a mut Cell {
    let surr_count = surr_indices(cells, offsets, index).len();
    let cell = &mut cells[index];

    if let &mut None = cell {
        *cell = Some(Cell::new(surr_count));
    }

    cell.as_mut().unwrap()
}

fn surr_indices(
    cells: &[Option<Cell>], 
    offsets: &HashSet<isize>,
    index: usize
) -> Vec<usize> {
    offsets.iter()
        .filter_map(|&offset| {
            let surr_index = index as isize + offset as isize;
            if (0..cells.len() as isize).contains(surr_index) {
                Some(surr_index as usize)
            } else {
                None
            }
        })
        .collect()
}

fn coords_to_index(coords: &[usize], dims: &[usize]) -> usize {
    coords.iter().zip(dims.iter())
        .fold(0, |acc, (&coord, &dim)| {
            (acc * dim) + coord
        })
}

fn index_to_coords(index: usize, dims: &[usize]) -> Vec<usize> {
    let mut coords: Vec<usize> = dims.iter()
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

fn surr_offsets(dims: &[usize]) -> HashSet<isize>{
    dims.iter()
        .scan(1, |state, &dim| {
            let acc_dim = *state;
            *state *= dim;
            Some(acc_dim)
        })
        .map(|acc_dim| (-(acc_dim as isize)..=acc_dim as isize).step_by(acc_dim))
        .multi_cartesian_product()
        .map(|offsets| offsets.into_iter().sum())
        .filter(|&offset| offset != 0)
        .collect()
}

#[test]
fn test_surr_offsets() {
    for (dims, expected) in vec![
        (vec![1, 1], vec![-2, -1, 1, 2]),
        (vec![3, 3], vec![-4, -3, -2, -1, 1, 2, 3, 4]),
        (vec![10, 10], vec![-11, -10, -9, -1, 1, 9, 10, 11]),
        (vec![2, 2, 2], vec![-7, -6, -5, -4, -3, -2, -1, 1, 2, 3, 4, 5, 6, 7]),
        (vec![4, 5, 6], vec![-25, -24, -23, -21, -20, -19, -17, -16, -15, -5, -4, -3, -1, 1, 3, 4, 5, 15, 16, 17, 19, 20, 21, 23, 24, 25]),
    ] {
        let exp_set: HashSet<isize> = expected.iter().map(|&i| i).collect();
        let actual = surr_offsets(&dims);

        let assert_msg = {
            let mut actual_vec: Vec<isize> = actual.iter().map(|&i| i).collect();
            actual_vec.sort();
            format!(
                "surr_offsets(&{:?}) ->\nexp:   {:?}\nactual:{:?}",
                dims,
                expected,
                actual_vec
            )
        };

        assert_eq!(exp_set, actual, "\n\n{}\n\n", assert_msg);
    }
}