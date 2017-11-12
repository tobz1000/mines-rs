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
	client: &'a Client<'a>,
	dims: Vec<i32>,
	arr: Vec<Option<Cell<'a>>>,
	surr_cells: HashMap<
		&'a Cell<'a>,
		HashSet<&'a Cell<'a>>
	>
}

impl<'a> GameGrid<'a> {
	pub fn new(client: &'a Client, dims: Vec<i32>) -> Self {
		let arr_size = dims.iter().fold(1, |a, b| a * b) as usize;
		let mut arr: Vec<Option<Cell>> = (0..arr_size).map(|_| None).collect();
		let mut surr_cells = HashMap::new();

		GameGrid { client, dims, arr, surr_cells }
	}

	pub fn cell(&mut self, coords: Vec<i32>) -> &Cell {
		let index = (0..coords.len()).fold(0, |acc, i| {
			(acc * self.dims[i]) + coords[i]
		}) as usize;

		if self.arr[index] == None {
			let cell = Cell::new(coords, self.client);
			self.arr[index] = Some(cell);
		}

		self.arr[index].as_ref().unwrap()
	}

	pub fn surr_cells(&self, cell: &Cell) -> &mut Vec<&mut Cell> {
		unimplemented!()
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

struct Client<'a> {
	grid: GameGrid<'a>,
}

struct Cell<'a> {
	coords: Vec<i32>,
	state: CellState,
	surrounding_changed: bool,
	unknown_surr_count_mine: i32,
	unknown_surr_count_empty: Option<i32>,
	client: &'a Client<'a>
}

impl<'a> Cell<'a> {
	pub fn new(coords: Vec<i32>, client: &'a Client) -> Self {
		Cell {
			coords,
			client,
			state: CellState::Unknown,
			surrounding_changed: false,
			unknown_surr_count_mine: 0,
			unknown_surr_count_empty: None,
		}
	}

	pub fn unknown_surr_count_mine(&self) -> i32 {
		self.unknown_surr_count_mine
	}

	pub fn unknown_surr_count_empty(&mut self) -> i32 {
		if self.unknown_surr_count_empty == None {
			self.unknown_surr_count_empty = Some(
				self.surr_cells().len() as i32
			);
		}

		self.unknown_surr_count_empty.unwrap()
	}

	pub fn set_unknown_surr_count_empty(&mut self, val: i32) {
		if val == 0 {
			for cell in self.surr_cells() {
				if cell.state == CellState::Unknown {
					cell.state = CellState::Mine;
				}
			}
		}
	}

	fn surr_cells(&self) -> &mut Vec<&mut Cell> {
		self.client.grid.surr_cells(self)
	}
}

impl<'a> PartialEq for Cell<'a> {
	fn eq(&self, other: &Cell) -> bool {
		self.coords == other.coords
	}
}

impl<'a> Eq for Cell<'a> {}

impl<'a> Hash for Cell<'a> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.coords.hash(state);
	}
}