use itertools::Itertools;

use std::collections::HashSet;

use coords::Coords;

pub struct GameGrid<C> {
    pub dims: Vec<usize>,
    pub cells: Vec<C>,
}

impl<C> GameGrid<C> {
    pub fn new<F: Fn(usize, HashSet<usize>) -> C>(
        dims: Vec<usize>,
        get_cell: F
    ) -> Self {
        let all_coords = dims.iter().map(|&d| 0..d).multi_cartesian_product();

        let cells = all_coords.enumerate().map(|(i, coords)| {
            get_cell(i, Coords(coords).surr_indices(&dims))
        }).collect();

        GameGrid { dims, cells }
    }
}
