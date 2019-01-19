use std::iter::repeat;
use rand::{Rng, SeedableRng};
use mersenne_twister::MT19937;
use itertools::{Itertools, iproduct};
#[cfg(feature = "rayon")] use rayon::iter::{ParallelIterator, IntoParallelIterator};
use serde_derive::{Deserialize, Serialize};

use crate::GameError;
use crate::server::{GameServer, GameState, GameSpec};
use crate::client::Client;

#[derive(Debug, Serialize, Deserialize)]
pub struct GameBatch<D, M> {
    pub count_per_spec: usize,
    pub dims_range: Vec<D>,
    pub mines_range: M,
    pub autoclear: bool,
    pub metaseed: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpecResult<I> {
    pub dims: Vec<usize>,
    pub mines: usize,
    pub played: usize,
    pub wins: usize,
    pub info: Vec<I>
}

/// `js_serializable` implementation for `SpecResult<()>`. Used by webapp, but must be defined in
/// the same module as `SpecResult`.
#[cfg(feature = "webapp")]
mod empty_spec_result_js_impl {
    use super::SpecResult;
    use stdweb::{js_serializable, __js_serializable_serde_boilerplate, __js_serializable_boilerplate};

    type EmptySpecResult = SpecResult<()>;

    js_serializable!( EmptySpecResult );
}

#[derive(Clone)]
struct GridSpec {
    spec_index: usize,
    dims: Vec<usize>,
    mines: usize
}

struct GameResult<I> {
    spec_index: usize,
    win: bool,
    info: I
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
    pub fn run<G: GameServer, I: Send>(
        self,
        new_game: impl Fn(GameSpec) -> Result<G, GameError> + Sync,
        use_game_result: impl Fn(G) -> I + Sync
    ) -> Result<Vec<SpecResult<I>>, GameError> {
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
                    wins: 0,
                    info: Vec::new()
                })
            }

            specs.push((i, spec));
        }

        #[cfg(feature = "rayon")]
        let specs_iter = specs.into_par_iter();

        #[cfg(not(feature = "rayon"))]
        let specs_iter = specs.into_iter();

        let results: Vec<Result<GameResult<I>, GameError>> = specs_iter
            .map(|(spec_index, spec)| {
                let mut game = new_game(spec)?;

                {
                    let mut client = Client::new(&mut game);
                    client.play()?;
                }

                let win = game.game_state() == GameState::Win;
                let info = use_game_result(game);

                Ok(GameResult { spec_index, win, info })
            })
            .collect();

        for result in results {
            let GameResult { spec_index, win, info } = result?;
            let spec_result = &mut spec_results[spec_index];

            spec_result.played += 1;

            if win {
                spec_result.wins += 1;
            }

            spec_result.info.push(info);
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

    pub fn into_serializable(self) -> GameBatch<Vec<usize>, Vec<usize>> {
        let GameBatch {
            count_per_spec,
            dims_range,
            mines_range,
            autoclear,
            metaseed
        } = self;
        let dims_range = dims_range.into_iter()
            .map(|i| i.into_iter().collect())
            .collect();
        let mines_range = mines_range.into_iter().collect();

        GameBatch {
            count_per_spec,
            dims_range,
            mines_range,
            autoclear,
            metaseed
        }
    }
}