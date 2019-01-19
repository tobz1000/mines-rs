use itertools::Itertools;
use std::collections::HashSet;
use std::convert::TryInto;
use std::iter::repeat;
use std::ops::{Index, IndexMut};

use crate::coords::Coords;
use crate::util::index_pair;

pub struct GameGrid<C>(Vec<Option<C>>);

#[derive(Clone, Copy, Debug)]
enum DimReg {
    Start,
    Mid,
    End,
}

impl<C> GameGrid<C> {
    pub fn new<F: Fn(usize, HashSet<usize>) -> C>(dims: &[usize], get_cell: F) -> Self {
        if dims.iter().any(|&d| d < 2) {
            panic!("All grid dimensions must be >= 2");
        }

        let size = dims.iter().fold(1, |s, &i| s * i);
        let mut cells = Vec::with_capacity(size);
        cells.resize_default(size);

        let cell_regions = repeat(
            [DimReg::Start, DimReg::Mid, DimReg::End]
                .iter()
                .map(Clone::clone),
        )
        .take(dims.len())
        .multi_cartesian_product();

        for cell_region in cell_regions {
            let offsets = region_offsets(&cell_region, dims);
            let region_coords = cell_region
                .iter()
                .zip(dims.iter())
                .map(|(&r, &d)| match r {
                    DimReg::Start => 0..1,
                    DimReg::Mid => 1..d - 1,
                    DimReg::End => d - 1..d,
                })
                .multi_cartesian_product();

            for coords in region_coords {
                let index = Coords(coords.clone()).to_index(dims);
                let surr = offsets
                    .iter()
                    .map(|&o| {
                        (index as isize + o)
                            .try_into()
                            .expect("Calculated negative cell index")
                    })
                    .collect();

                cells[index] = Some(get_cell(index, surr));
            }
        }

        GameGrid(cells)
    }

    pub fn cell_pair(&mut self, ia: usize, ib: usize) -> (&mut C, &mut C) {
        let (a, b) = index_pair(self.0.as_mut_slice(), ia, ib);

        (a.as_mut().unwrap(), b.as_mut().unwrap())
    }

    pub fn iter(&self) -> impl Iterator<Item = &C> {
        self.0.iter().map(|c| c.as_ref().unwrap())
    }
}

impl<C> Index<usize> for GameGrid<C> {
    type Output = C;

    fn index(&self, index: usize) -> &Self::Output {
        self.0[index].as_ref().unwrap()
    }
}

impl<C> IndexMut<usize> for GameGrid<C> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.0[index].as_mut().unwrap()
    }
}

// Does not account for dims where some dimension is of size 1; will produce
// incorrect (overflowing) offsets in this case.
fn region_offsets(cell_region: &[DimReg], dims: &[usize]) -> Vec<isize> {
    cell_region
        .iter()
        .map(|&dim_region| match dim_region {
            DimReg::Start => 0..=1,
            DimReg::Mid => -1..=1,
            DimReg::End => -1..=0,
        })
        .multi_cartesian_product()
        .filter_map(|coord_offs| {
            let index_offset = coord_offs
                .iter()
                .zip(dims.iter())
                .fold(0, |acc, (&o, &d)| acc * d as isize + o);

            // Do not include co-ordinate itself in offsets
            if index_offset == 0 {
                None
            } else {
                Some(index_offset)
            }
        })
        .collect()
}
