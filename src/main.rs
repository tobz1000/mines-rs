#![feature(inclusive_range_syntax)]
#![feature(conservative_impl_trait)]
#![feature(vec_resize_default)]
#![feature(iterator_try_fold)]

#[macro_use] extern crate serde_derive;
#[macro_use] extern crate itertools;
extern crate mersenne_twister;
extern crate rand;

extern crate serde_json;
extern crate tokio_core;

mod server;
mod coords;
mod game_grid;
mod client;
mod game_batch;
mod util;

use game_batch::GameBatch;

fn main() {
    let batch = GameBatch {
        count_per_spec: 2000,
        dims_range: vec![6..=6, 6..=6],
        mines_range: 0..19,
        autoclear: true,
        metaseed: 0xB16B0B5
    };

    let results = batch.clone().run().unwrap();

    for ((dims, mines), wins) in results {
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
}