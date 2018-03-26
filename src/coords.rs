use itertools::Itertools;

use std::collections::HashSet;
use std::iter::repeat;
use std::convert::TryInto;

pub type Coords = Vec<usize>;

pub fn surr_indices(coords: &Coords, dims: &Coords) -> HashSet<usize> {
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

pub fn coords_to_index(coords: &Coords, dims: &Coords) -> usize {
    coords.iter().zip(dims.iter())
        .fold(0, |acc, (&coord, &dim)| (acc * dim) + coord)
}

pub fn index_to_coords(index: usize, dims: &Coords) -> Coords {
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
        (vec![0, 0], vec![1, 1], vec![]),
        (vec![1, 0], vec![2, 2], vec![0, 1, 3])
    ] {
        let exp_set = exp.into_iter().collect();
        assert_eq!(surr_indices(&coords, &dims), exp_set);
    }
}