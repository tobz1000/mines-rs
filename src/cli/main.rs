#[macro_use] extern crate structopt;
extern crate chrono;
extern crate mines_rs;

mod options;

use self::chrono::Utc;
use mines_rs::{
    GameBatch,
    SpecResult,
    JsServerWrapper,
    NativeServer,
    mongodb_connector
};
use options::{RunBatchOptions, ServerType};
use structopt::StructOpt;

fn main() {
    let RunBatchOptions {
        count_per_spec,
        dims_range,
        mines_range,
        metaseed,
        server_type,
        save_to_db
    } = RunBatchOptions::from_args();

    let batch = GameBatch {
        count_per_spec,
        dims_range,
        mines_range,
        autoclear: true,
        metaseed,
    };

    let start = Utc::now();

    let results = match (server_type, save_to_db) {
        (ServerType::Native, false) => {
            batch.run(|spec| NativeServer::new(spec, false), |_game| ()).unwrap()
        },
        (ServerType::Native, true) => {
            batch.run(
                |spec| NativeServer::new(spec, true),
                |game| { mongodb_connector::insert_game(&game).unwrap(); }
            ).unwrap()
        },
        (ServerType::Js, false) => {
            batch.run(JsServerWrapper::new, |_game| ()).unwrap()
        },
        (ServerType::Js, true) => {
            panic!("save_to_db command line option is invalid for JS server")
        },
    };

    let game_count = results.len() * count_per_spec;

    println!("Dims\t\tMines\tWins/Played");

    for SpecResult { dims, mines, wins, played, info: _ } in results {
        let win_perc = wins as f64 * 100f64 / count_per_spec as f64;
        println!(
            "{:?}\t{}:\t{}/{}\t({:.0}%)",
            dims,
            mines,
            wins,
            played,
            win_perc
        );
    }

    let stop = Utc::now();
    let dur_us = (stop - start).num_microseconds().unwrap();
    let dur_s = dur_us as f64 / 1_000_000_f64;
    let avg_us = dur_us / game_count as i64;

    println!("Time: {:.2}s (avg {}µs/game/core)", dur_s, avg_us);
}