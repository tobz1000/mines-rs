use itertools::Itertools;

use std::collections::HashSet;

use coords::Coords;

pub struct GameGrid<C> {
    pub dims: Vec<usize>,
    pub cells: Vec<C>,
}

impl<C> GameGrid<C> {
    pub fn new(
        dims: Vec<usize>,
        get_cell: fn(index: usize, surr: HashSet<usize>) -> C
    ) -> Self {
        let all_coords = dims.iter().map(|&d| 0..d).multi_cartesian_product();

        let cells = all_coords.enumerate().map(|(i, coords)| {
            get_cell(i, Coords(coords).surr_indices(&dims))
        }).collect();

        GameGrid { dims, cells }
    }
}
