#![feature(vec_resize_default)]
#![feature(iterator_try_fold)]
#![feature(try_from)]

#[macro_use] extern crate serde_derive;
#[macro_use] extern crate itertools;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate structopt;
extern crate chrono;

#[cfg(test)] #[macro_use] extern crate quickcheck;

mod server;
mod coords;
mod game_grid;
mod client;
mod game_batch;
mod util;
mod options;

use std::error::Error;
use self::chrono::Utc;
use game_batch::SpecResult;
use options::Options;

type GameError = Box<dyn Error + Sync + Send>;

fn main() {
    use structopt::StructOpt;
    let batch = Options::from_args().into_game_batch();

    let start = Utc::now();
    let results = batch.clone().run_native(false).unwrap();
    let game_count = results.len() * batch.count_per_spec;

    println!("Dims\t\tMines\tWins/Played");

    for SpecResult { dims, mines, wins, .. } in results {
        let win_perc = wins as f64 * 100f64 / batch.count_per_spec as f64;
        println!(
            "{:?}\t{}:\t{}/{}\t({:.0}%)",
            dims,
            mines,
            wins,
            batch.count_per_spec,
            win_perc
        );
    }

    let stop = Utc::now();
    let dur_us = (stop - start).num_microseconds().unwrap();
    let dur_s = dur_us as f64 / 1_000_000_f64;
    let avg_us = dur_us / game_count as i64;

    println!("Time: {:.2}s (avg {}µs/game)", dur_s, avg_us);
}