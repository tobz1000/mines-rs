extern crate rand;
extern crate mersenne_twister;
extern crate itertools;

use std::error::Error;
use self::rand::{Rng, SeedableRng};
use self::mersenne_twister::MT19937;
use self::itertools::Itertools;
use server::{NativeServer, GameState};
use client::Client;

#[derive(Clone)]
pub struct GameBatch<D, M>
    where D: IntoIterator<Item=usize>,
          <D as IntoIterator>::IntoIter: Clone,
          M: IntoIterator<Item=usize>,
          <M as IntoIterator>::IntoIter: Clone,
{
    pub count_per_spec: usize,
    pub dims_range: Vec<D>,
    pub mines_range: M,
    pub autoclear: bool,
    pub metaseed: u32,
}

impl<D, M> GameBatch<D, M>
    where D: IntoIterator<Item=usize>,
          <D as IntoIterator>::IntoIter: Clone,
          M: IntoIterator<Item=usize>,
          <M as IntoIterator>::IntoIter: Clone,
{
    pub fn run(self) -> Result<
        Vec<((Vec<usize>, usize), usize)>,
        Box<Error>
    > {
        let GameBatch {
            count_per_spec,
            dims_range,
            mines_range,
            autoclear,
            metaseed
        } = self;

        let mut ret = Vec::new();
        let mut rng: MT19937 = SeedableRng::from_seed(metaseed);
        let all_dims = dims_range.into_iter().multi_cartesian_product();

        for (dims, mines) in iproduct!(all_dims, mines_range) {
            let size = dims.iter().fold(1, |s, &d| s * d);
            if size <= mines { continue; }

            let wins = rng.gen_iter().take(count_per_spec)
                .try_fold(0, |wins, seed| -> Result<usize, Box<Error>> {
                    let win = Self::single_game(
                        dims.clone(),
                        mines,
                        seed,
                        autoclear
                    ).unwrap() == GameState::Win;

                    Ok(if win { wins + 1 } else { wins })
                }).unwrap();

            ret.push(((dims, mines), wins));
        }

        Ok(ret)
    }

    fn single_game(
        dims: Vec<usize>,
        mines: usize,
        seed: u32,
        autoclear: bool,
    ) -> Result<GameState, Box<Error>> {
        let server = NativeServer::new(dims, mines, Some(seed), autoclear);
        let mut client = Client::new(server);

        client.play()
    }
}