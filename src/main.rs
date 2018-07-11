#![feature(vec_resize_default)]
#![feature(iterator_try_fold)]
#![feature(try_from)]

#[macro_use] extern crate serde_derive;
#[macro_use] extern crate itertools;
#[macro_use] extern crate lazy_static;
extern crate chrono;

mod server;
mod coords;
mod game_grid;
mod client;
mod game_batch;
mod util;

use self::chrono::Utc;
use game_batch::{GameBatch, SpecResult};

fn main() {
    let batch = GameBatch {
        count_per_spec: 20,
        dims_range: vec![6..=6, 6..=6],
        mines_range: 0..10,
        autoclear: true,
        metaseed: 0xB16B0B5
    };

    let start = Utc::now();
    let results = batch.clone().run_native().unwrap();
    let game_count = results.len() * batch.count_per_spec;

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