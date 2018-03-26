use itertools::Itertools;

use coords::Coords;
use client::Cell;

pub struct GameGrid {
    pub dims: Vec<usize>,
    pub cells: Vec<Cell>,
}

impl GameGrid {
    pub fn new(dims: Vec<usize>) -> Self {
        let all_coords = dims.iter().map(|&d| 0..d).multi_cartesian_product();

        let cells = all_coords.enumerate().map(|(i, coords)| {
            Cell::new(i, Coords(coords).surr_indices(&dims))
        }).collect();

        GameGrid { dims, cells }
    }
}
