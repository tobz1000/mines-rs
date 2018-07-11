extern crate rand;
extern crate mersenne_twister;
extern crate itertools;
extern crate mongodb;

use std::error::Error;
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

struct Spec {
    dims: Vec<usize>,
    mines: usize,
    seed: u32,
    autoclear: bool,
}

lazy_static! {
    static ref DB_CONNECTION: Database = {
        let client = mongodb::ThreadedClient::connect("localhost", 27017).unwrap();
        Database::open(client, "test", None, None)
    };
}

fn new_native_game(spec: Spec) -> Result<NativeServer, Box<Error>> {
    let Spec { dims, mines, seed, autoclear } = spec;

    Ok(NativeServer::new(
        dims,
        mines,
        Some(seed),
        autoclear,
        Some(DB_CONNECTION.clone())
    ))
}

fn new_json_server_game(spec: Spec) -> Result<JsonServerWrapper, Box<Error>> {
    let Spec { dims, mines, seed, autoclear } = spec;

    JsonServerWrapper::new_game(dims, mines, Some(seed), autoclear)
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

    fn run<F, G>(self, new_game: F) -> Result<Vec<SpecResult>, Box<dyn Error>>
        where G: GameServer,
              F: Fn(Spec) -> Result<G, Box<dyn Error>>,
    {
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
                    let game = new_game(Spec {
                        dims: dims.clone(),
                        mines,
                        seed,
                        autoclear
                    })?;

                    let mut client = Client::new(game);
                    let win = client.play()? == GameState::Win;

                    Ok(if win { wins + 1 } else { wins })
                })?;

            ret.push(SpecResult {
                dims: dims.clone(),
                mines,
                played: count_per_spec,
                wins
            });
        }

        Ok(ret)
    }
}