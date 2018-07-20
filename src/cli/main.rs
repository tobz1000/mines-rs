
#[macro_use] extern crate structopt;
extern crate chrono;
extern crate mines_rs;

mod options;

use self::chrono::Utc;
use mines_rs::SpecResult;
use options::Options;

fn main() {
    use structopt::StructOpt;
    let batch = Options::from_args().into_game_batch();
    let count_per_spec = batch.count_per_spec;

    let start = Utc::now();
    let results = batch.run().unwrap();
    let game_count = results.len() * count_per_spec;

    println!("Dims\t\tMines\tWins/Played");

    for SpecResult { dims, mines, wins, .. } in results {
        let win_perc = wins as f64 * 100f64 / count_per_spec as f64;
        println!(
            "{:?}\t{}:\t{}/{}\t({:.0}%)",
            dims,
            mines,
            wins,
            count_per_spec,
            win_perc
        );
    }

    let stop = Utc::now();
    let dur_us = (stop - start).num_microseconds().unwrap();
    let dur_s = dur_us as f64 / 1_000_000_f64;
    let avg_us = dur_us / game_count as i64;

    println!("Time: {:.2}s (avg {}µs/game/core)", dur_s, avg_us);
}