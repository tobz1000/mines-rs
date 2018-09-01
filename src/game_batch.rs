extern crate rand;
extern crate mersenne_twister;
extern crate itertools;
extern crate rayon;

use std::iter::repeat;
use self::rand::{Rng, SeedableRng};
use self::mersenne_twister::MT19937;
use self::itertools::Itertools;
use self::rayon::iter::{ParallelIterator, IntoParallelIterator};
use ::GameError;
use server::{GameServer, GameState, GameSpec};
use client::Client;

pub struct GameBatch<D, M> {
    pub count_per_spec: usize,
    pub dims_range: Vec<D>,
    pub mines_range: M,
    pub autoclear: bool,
    pub metaseed: u32,
}

pub struct SpecResult {
    pub dims: Vec<usize>,
    pub mines: usize,
    pub played: usize,
    pub wins: usize
}

#[derive(Clone)]
struct GridSpec {
    spec_index: usize,
    dims: Vec<usize>,
    mines: usize
}

struct GameResult {
    spec_index: usize,
    win: bool
}

struct GameSpecs<G: Iterator<Item=GridSpec>, R: Rng> {
    grid_specs: G,
    autoclear: bool,
    rng: R,
}

impl<G: Iterator<Item=GridSpec>, R: Rng> Iterator for GameSpecs<G, R> {
    type Item = (usize, GameSpec);

    fn next(&mut self) -> Option<Self::Item> {
        let GameSpecs { grid_specs, autoclear, rng } = self;
        let GridSpec { spec_index, dims, mines } = grid_specs.next()?;
        let seed = rng.next_u32();

        Some((spec_index, GameSpec { dims, mines, seed, autoclear: *autoclear }))
    }
}

impl<D, M> GameBatch<D, M>
    where D: IntoIterator<Item=usize>,
          <D as IntoIterator>::IntoIter: Clone,
          M: IntoIterator<Item=usize>,
          <M as IntoIterator>::IntoIter: Clone
{
    pub fn run<G: GameServer>(
        self,
        config: G::Config
    ) -> Result<Vec<SpecResult>, GameError> {
        let mut specs = Vec::new();
        let mut spec_results = Vec::new();

        for (i, spec) in self.game_specs() {
            if i > spec_results.len() {
                panic!("spec_index out of order");
            }

            if i == spec_results.len() {
                spec_results.push(SpecResult {
                    dims: spec.dims.clone(),
                    mines: spec.mines,
                    played: 0,
                    wins: 0
                })
            }

            specs.push((i, spec));
        }

        let results: Vec<Result<GameResult, GameError>> = specs.into_par_iter()
            .map(|(spec_index, spec)| {
                let game = G::new(spec, config.clone())?;

                let mut client = Client::new(game);

                let win = client.play()? == GameState::Win;

                Ok(GameResult { win, spec_index })
            })
            .collect();

        for result in results {
            let GameResult { spec_index, win } = result?;

            spec_results[spec_index].played += 1;
            if win {
                spec_results[spec_index].wins += 1;
            }
        }

        Ok(spec_results)
    }

    fn game_specs(self) -> GameSpecs<impl Iterator<Item=GridSpec>, MT19937> {
        let GameBatch {
            count_per_spec,
            dims_range,
            mines_range,
            autoclear,
            metaseed,
        } = self;

        let all_dims = dims_range.into_iter().multi_cartesian_product();

        let grid_specs = iproduct!(all_dims, mines_range)
            .filter(|(dims, mines)| {
                let size = dims.iter().fold(1, |s, &d| s * d);
                size > *mines
            })
            .enumerate()
            .flat_map(move |(spec_index, (dims, mines))| {
                repeat(GridSpec { spec_index, dims, mines })
                    .take(count_per_spec)
            });

        let rng: MT19937 = SeedableRng::from_seed(metaseed);

        GameSpecs { grid_specs, autoclear, rng }
    }
}