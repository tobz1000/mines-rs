#![cfg(test)]

extern crate itertools;

// use std::hash::{Hash, Hasher};
// use std::collections::{HashSet, HashMap};
use itertools::Itertools;

#[derive(PartialEq, Eq)]
enum CellState {
	Unknown,
	ToClear,
	Empty,
	Mine
}

struct GameGrid {
	dims: Vec<i32>,
	arr: Vec<Option<Cell>>,
	surr_cell_offsets: Vec<isize>
}

impl GameGrid {
	fn new(dims: Vec<i32>) -> Self {
		let arr_size = dims.iter().fold(1, |a, b| a * b) as usize;
		let arr: Vec<Option<Cell>> = (0..arr_size).map(|_| None).collect();

		let surr_cell_offsets = dims.iter()
			.scan(0, |state, &dim| {
				*state *= dim as isize;
				Some(*state)
			})
			.map(|acc_dim| (-acc_dim..(acc_dim + 1)).step_by(acc_dim as usize))
			.multi_cartesian_product()
			.map(|offsets| offsets.into_iter().sum())
			.filter(|&offset| offset == 0)
			.collect();

		GameGrid { dims, arr, surr_cell_offsets }
	}

	fn coords_to_index(dims: Vec<i32>, coords: Vec<i32>) -> usize {
		coords.into_iter().zip(dims)
			.fold(0, |acc, (coord, dim)| {
				(acc * dim) + coord
			}) as usize
	}

	fn surr_indices(&self, index: usize) -> Vec<usize> {
		self.surr_cell_offsets.iter()
			.filter_map(|&offset| {
				let surr_index = index as isize + offset;
				if (0..self.arr.len() as isize).contains(surr_index) {
					Some(surr_index as usize)
				} else {
					None
				}
			})
			.collect()
	}

	fn cell_from_coords(&mut self, coords: Vec<i32>) -> &mut Cell {
		let dims = self.dims.clone();
		self.cell_from_index(Self::coords_to_index(dims, coords))
	}

	fn cell_from_index<'a, 'b>(&'b mut self, index: usize) -> &'a mut Cell {
		let cell = &mut self.arr[index];

		if let None = *cell {
			let surr_indices = self.surr_indices(index);
			*cell = Some(Cell::new(surr_indices));
		}

		(*cell).as_mut().unwrap()
	}

	fn surr_cells<'a>(&'a mut self, cell: &Cell) -> Vec<&'a mut Cell> {
		cell.surr_indices.iter()
			.map(|&index| self.cell_from_index::<'a>(index))
			.collect()
	}

	fn unknown_surr_count_mine(&self, cell: &Cell) -> i32 {
		cell.unknown_surr_count_mine
	}

	fn set_unknown_surr_count_mine(&self, cell: &Cell, val: i32) {
		if (val == 0) & (cell.state == CellState::Empty) {
			for surr_cell in self.surr_cells(cell) {
				if surr_cell.state == CellState::Empty {
					surr_cell.state = CellState::ToClear;
				}
			}
		}

		cell.unknown_surr_count_mine = val;
	}

	fn unknown_surr_count_empty(&self, cell: &Cell) -> i32 {
		cell.unknown_surr_count_empty
	}

	fn set_unknown_surr_count_empty(&self, cell: &Cell, val: i32) {
		if val == 0 {
			for surr_cell in self.surr_cells(cell) {
				if surr_cell.state == CellState:: Unknown {
					surr_cell.state = CellState::Mine;
				}
			}
		}

		cell.unknown_surr_count_mine = val;
	}
}

struct Client {
	grid: GameGrid,
}

struct Cell {
	// index: usize,
	surr_indices: Vec<usize>,
	state: CellState,
	surrounding_changed: bool,
	unknown_surr_count_mine: i32,
	unknown_surr_count_empty: i32
}

impl Cell {
	pub fn new(surr_indices: Vec<usize>) -> Self {
		Cell {
			surr_indices,
			state: CellState::Unknown,
			surrounding_changed: false,
			unknown_surr_count_mine: 0,
			unknown_surr_count_empty: surr_indices.len() as i32,
		}
	}
}

// impl PartialEq for Cell {
// 	fn eq(&self, other: &Cell) -> bool {
// 		self.coords == other.coords
// 	}
// }

// impl Eq for Cell {}

// impl Hash for Cell {
// 	fn hash<H: Hasher>(&self, state: &mut H) {
// 		self.coords.hash(state);
// 	}
// }

// #[test]
// fn test_surr_coords() {
// 	let surr = GameGrid::_surr_coords(
// 		&vec![10, 10],
// 		&vec![9, 5]
// 	);

// 	let expected = vec![
// 		vec![8, 4],
// 		vec![8, 5],
// 		vec![8, 6],
// 		vec![9, 4],
// 		vec![9, 6],
// 	];
	
// 	assert_eq!(surr, expected);
// }
