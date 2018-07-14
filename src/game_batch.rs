extern crate rand;
extern crate mersenne_twister;
extern crate itertools;
extern crate mongodb;

use std::error::Error;
use std::iter::repeat;
use self::rand::{Rng, SeedableRng};
use self::mersenne_twister::MT19937;
use self::itertools::Itertools;
use self::mongodb::db::{Database, ThreadedDatabase};
use server::{NativeServer, JsonServerWrapper, GameServer, GameState};
use client::Client;

#[derive(Clone)]
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

struct GameSpec {
    dims: Vec<usize>,
    mines: usize,
    seed: u32,
    autoclear: bool,
}

#[derive(Clone)]
struct GridSpec {
    spec_index: usize,
    dims: Vec<usize>,
    mines: usize
}

lazy_static! {
    static ref DB_CONNECTION: Database = {
        let client = mongodb::ThreadedClient::connect("localhost", 27017).unwrap();
        Database::open(client, "test", None, None)
    };
}

fn new_native_game(spec: GameSpec) -> Result<NativeServer, Box<Error>> {
    let GameSpec { dims, mines, seed, autoclear } = spec;

    Ok(NativeServer::new(
        dims,
        mines,
        Some(seed),
        autoclear,
        Some(DB_CONNECTION.clone())
    ))
}

fn new_json_server_game(spec: GameSpec) -> Result<JsonServerWrapper, Box<Error>> {
    let GameSpec { dims, mines, seed, autoclear } = spec;

    JsonServerWrapper::new_game(dims, mines, Some(seed), autoclear)
}

struct GameSpecs<G, R>
    where G: Iterator<Item=GridSpec>,
          R: Rng
{
    grid_specs: G,
    autoclear: bool,
    rng: R,
}

impl<G, R> Iterator for GameSpecs<G, R>
    where G: Iterator<Item=GridSpec>,
          R: Rng
{
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
          <M as IntoIterator>::IntoIter: Clone,
{
    #[allow(dead_code)]
    pub fn run_json_server(self) -> Result<Vec<SpecResult>, Box<dyn Error>> {
        self.run(new_json_server_game)
    }

    #[allow(dead_code)]
    pub fn run_native(self) -> Result<Vec<SpecResult>, Box<dyn Error>> {
        self.run(new_native_game)
    }

    fn game_specs(self) -> GameSpecs<impl Iterator<Item=GridSpec>, MT19937> {
        let GameBatch {
            count_per_spec,
            dims_range,
            mines_range,
            autoclear,
            metaseed
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

    fn run<F, G>(self, new_game: F) -> Result<Vec<SpecResult>, Box<dyn Error>>
        where G: GameServer,
              F: Fn(GameSpec) -> Result<G, Box<dyn Error>>,
    {
        let mut results = Vec::new();

        for (i, spec) in self.game_specs() {
            while i >= results.len() {
                results.push(SpecResult {
                    dims: spec.dims.clone(),
                    mines: spec.mines,
                    played: 0,
                    wins: 0
                })
            }

            let game = new_game(spec)?;
            let mut client = Client::new(game);
            let win = client.play()? == GameState::Win;

            results[i].played += 1;
            if win {
                results[i].wins += 1;
            }
        }

        Ok(results)
    }
}