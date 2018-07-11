use std::fmt;
use std::convert::{TryFrom, TryInto};

#[derive(Clone, Serialize, Deserialize)]
pub struct Coords<T = usize>(pub Vec<T>);

impl<T: Copy> Coords<T>
    where T: TryFrom<usize> + TryInto<usize>,
          <T as TryFrom<usize>>::Error: fmt::Debug,
          <T as TryInto<usize>>::Error: fmt::Debug,
{
    pub fn from_index(index: usize, dims: &[usize]) -> Self {
        Coords(dims.iter().scan(index, |remainder, &dim| {
            let coord = T::try_from(*remainder % dim).unwrap();
            *remainder /= dim;
            Some(coord)
        }).collect())
    }

    pub fn to_index(&self, dims: &[usize]) -> usize {
        self.0.iter().zip(dims.iter())
            .fold(0, |acc, (&coord, &dim)| (acc * dim) + coord.try_into().unwrap())
    }
}

impl<T: fmt::Debug> fmt::Debug for Coords<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}