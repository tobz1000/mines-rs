#![cfg(test)]

extern crate itertools;

use std::hash::{Hash, Hasher};
use std::collections::{HashSet, HashMap};
use itertools::Itertools;

#[derive(PartialEq, Eq)]
enum CellState {
	Unknown,
	ToClear,
	Empty,
	Mine
}

struct GameGrid<'a> {
	dims: Vec<i32>,
	arr: Vec<Option<Cell>>,
	surr_cells: HashMap<&'a Cell, HashSet<&'a mut Cell>>
}

impl<'a> GameGrid<'a> {
	pub fn new(dims: Vec<i32>) -> Self {
		let arr_size = dims.iter().fold(1, |a, b| a * b) as usize;
		let mut arr: Vec<Option<Cell>> = (0..arr_size).map(|_| None).collect();
		let mut surr_cells = HashMap::with_capacity(arr_size);

		GameGrid { dims, arr, surr_cells }
	}

	pub fn cell(&'a mut self, coords: Vec<i32>) -> &'a mut Cell {
		let index = coords.iter().zip(self.dims.iter())
			.fold(0, |acc, (&coord, &dim)| {
				(acc * dim) + coord
			}) as usize;

		let mut cell = &mut self.arr[index];

		if *cell == None {
			*cell = Some(Cell::new(coords));
		}

		(*cell).as_mut().unwrap()
	}

	pub fn surr_cells(&'a mut self, cell: &'a Cell) -> &'a HashSet<&'a mut Cell> {
		match self.surr_cells.get(cell) {
			Some(surr_cells) => surr_cells,
			None => {
				let surr_cells: HashSet<&'a mut Cell> = self
					.surr_coords(&cell.coords)
					.into_iter()
					.map(|coords| self.cell(coords))
					.collect();

				self.surr_cells.insert(cell, surr_cells);
				&surr_cells
			}
		}
	}

	fn _surr_coords(dims: &Vec<i32>, coords: &Vec<i32>) -> Vec<Vec<i32>> {
		let offsets = (0..coords.len())
			.map(|_| -1..2)
			// Get all compbinations of coordinate offsets of -1, 0, 1
			.multi_cartesian_product()
			// Skip origin coords
			.filter(|offset| !offset.iter().all(|&o| o == 0)); 
		
		let surr_coords = offsets
			.map(|offset| {
				offset.iter().zip(coords).map(|(&o, &c)| o + c).collect()
			})
			.filter(|surr_coords| {
				// Remove out-of-bounds coords
				dims.iter().zip(surr_coords).all(|(&d, &s)| {
					(d > s) & (s >= 0)
				})
			});
		
		surr_coords.collect()
	}

	fn surr_coords(&self, coords: &Vec<i32>) -> Vec<Vec<i32>> {
		Self::_surr_coords(&self.dims, &coords)
	}

	pub fn unknown_surr_count_mine(&self, cell: &Cell) -> i32 {
		cell.unknown_surr_count_mine
	}

	pub fn set_unknown_surr_count_mine(&self, cell: &Cell, val: i32) {
		if (val == 0) & (cell.state == CellState::Empty) {
			for surr_cell in self.surr_cells(cell) {
				if surr_cell.state == CellState::Empty {
					surr_cell.state = CellState::ToClear;
				}
			}
		}

		cell.unknown_surr_count_mine = val;
	}

	pub fn unknown_surr_count_empty(&self, cell: &Cell) -> i32 {
		match cell.unknown_surr_count_empty {
			Some(count) => count,
			None => {
				let count = self.surr_cells(cell).len() as i32;
				cell.unknown_surr_count_empty = Some(count);
				count
			}
		}
	}

	pub fn set_unknown_surr_count_empty(&self, cell: &Cell, val: i32) {
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

struct Client<'a> {
	grid: GameGrid<'a>,
}

struct Cell {
	coords: Vec<i32>,
	state: CellState,
	surrounding_changed: bool,
	unknown_surr_count_mine: i32,
	unknown_surr_count_empty: Option<i32>
}

impl Cell {
	pub fn new(coords: Vec<i32>) -> Self {
		Cell {
			coords,
			state: CellState::Unknown,
			surrounding_changed: false,
			unknown_surr_count_mine: 0,
			unknown_surr_count_empty: None,
		}
	}
}

impl PartialEq for Cell {
	fn eq(&self, other: &Cell) -> bool {
		self.coords == other.coords
	}
}

impl Eq for Cell {}

impl Hash for Cell {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.coords.hash(state);
	}
}

#[test]
fn test_surr_coords() {
	let surr = GameGrid::_surr_coords(
		&vec![10, 10],
		&vec![9, 5]
	);

	let expected = vec![
		vec![8, 4],
		vec![8, 5],
		vec![8, 6],
		vec![9, 4],
		vec![9, 6],
	];
	
	assert_eq!(surr, expected);
}
