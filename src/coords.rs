use std::fmt;

#[derive(Clone, Serialize, Deserialize)]
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
}

impl fmt::Debug for Coords {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}