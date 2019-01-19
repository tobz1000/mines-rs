use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::convert::{TryFrom, TryInto};
use std::fmt;

#[derive(Clone)]
pub struct Coords<T = usize>(pub Vec<T>);

impl<T> Coords<T>
where
    T: Default + Copy + TryFrom<usize> + TryInto<usize>,
    <T as TryFrom<usize>>::Error: fmt::Debug,
    <T as TryInto<usize>>::Error: fmt::Debug,
{
    pub fn from_index(mut index: usize, dims: &[usize]) -> Self {
        let mut coords = vec![Default::default(); dims.len()];

        for (c, d) in coords.iter_mut().zip(dims).rev() {
            *c = T::try_from(index % d).unwrap();
            index /= d;
        }

        Coords(coords)
    }

    pub fn to_index(&self, dims: &[usize]) -> usize {
        self.0
            .iter()
            .zip(dims.iter())
            .fold(0, |acc, (&coord, &dim)| {
                (acc * dim) + coord.try_into().unwrap()
            })
    }
}

impl<T: fmt::Debug> fmt::Debug for Coords<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: Serialize> Serialize for Coords<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Coords<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let vec = Vec::<T>::deserialize(deserializer)?;
        Ok(Coords(vec))
    }
}

#[cfg(test)]
mod test {
    use quickcheck::TestResult;
    use std::mem::size_of;

    use crate::coords::Coords;

    quickcheck! {
        fn test_coords(i: usize, dims: Vec<u8>) -> TestResult {
            const DIMS_MAX: usize = size_of::<usize>() / size_of::<u8>();

            let dims: Vec<usize> = dims.into_iter()
                .filter(|&d| d > 0)
                .map(|d| d as usize)
                .take(DIMS_MAX)
                .collect();

            let index_max: usize = dims.iter().product();

            if i >= index_max {
                return TestResult::discard();
            }

            let coords: Coords<usize> = Coords::from_index(i, &dims);
            let derived_index = coords.to_index(&dims);

            println!("{:?}/{:?}: {} == {}?", dims, coords, i, derived_index);

            TestResult::from_bool(i == derived_index)
        }
    }
}
