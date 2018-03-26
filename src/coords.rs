use itertools::Itertools;

use std::collections::HashSet;
use std::iter::repeat;
use std::convert::TryInto;
use std::fmt;

#[derive(Serialize, Deserialize)]
pub struct Coords(pub Vec<usize>);

impl Coords {
    pub fn from_index(index: usize, dims: &[usize]) -> Self {
        let mut coords = vec![0; dims.len()];
        let mut remainder = index;

        for i in (0..dims.len()).rev() {
            coords[i] = remainder % dims[i];
            remainder /= dims[i];
        }

        Coords(coords)
    }

    pub fn to_index(&self, dims: &[usize]) -> usize {
        self.0.iter().zip(dims.iter())
            .fold(0, |acc, (&coord, &dim)| (acc * dim) + coord)
    }

    pub fn surr_indices(&self, dims: &[usize]) -> HashSet<usize> {
        let offsets = repeat(-1..=1).take(dims.len())
            .multi_cartesian_product()
            .filter(|offset| offset.iter().any(|&c| c != 0));

        let surr_coords = offsets.filter_map(|offset| -> Option<Coords> {
            let mut surr = vec![];

            for ((o, &c), &d) in offset.into_iter()
                .zip(self.0.iter())
                .zip(dims.iter())
            {
                let s = (o + (c as isize)).try_into().ok()?;
                if s >= d { return None; }
                surr.push(s);
            }

            Some(Coords(surr))
        });

        surr_coords.map(|surr| surr.to_index(dims)).collect()
    }
}

impl fmt::Debug for Coords {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
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
        assert_eq!(Coords(coords).surr_indices(&dims), exp_set);
    }
}