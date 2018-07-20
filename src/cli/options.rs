extern crate mines_rs;

use std::iter::StepBy;
use std::error::Error;
use std::ops::RangeInclusive;
use mines_rs::GameBatch;

type RangeOpt = StepBy<RangeInclusive<usize>>;

fn parse_range(s: &str) -> Result<RangeOpt, Box<dyn Error>> {
    let mut sub_args = s.split("..");

    let lower = sub_args.next().unwrap().parse()?;
    let upper = match sub_args.next() {
        Some(upper) => upper.parse()?,
        None => { return Ok((lower..=lower).step_by(1)); }
    };
    let step = match sub_args.next() {
        Some(step) => step.parse()?,
        None => { return Ok((lower..=upper).step_by(1)); }
    };

    Ok((lower..=upper).step_by(step))
}

fn parse_mines_range(s: &str) -> Result<RangeOpt, &str> {
    parse_range(s).or(Err("Mines range should be of the form start[..end][..step], e.g. `10` `10..50..5`"))
}

fn parse_dims_range(s: &str) -> Result<RangeOpt, &str> {
    parse_range(s).or(Err(
        "Dims should be in the form start[..end][..step][,...], e.g. `15` `2..5,10..30..10`"
    ))
}

fn parse_seed(s: &str) -> Result<u32, &str> {
    s.parse().or(Err(
        "Seed should be positive number less than U32MAX, e.g. `56023`"
    ))
}

#[derive(StructOpt, Debug)]
pub struct Options {
    #[structopt(short = "c", default_value ="100")]
    count_per_spec: usize,

    #[structopt(
        short = "d",
        parse(try_from_str = "parse_dims_range"),
        raw(use_delimiter = "true"),
        default_value = "20,20"
    )]
    dims_range: Vec<RangeOpt>,

    #[structopt(
        short = "m",
        parse(try_from_str = "parse_mines_range"),
        default_value = "10..50..5"
    )]
    mines_range: RangeOpt,

    #[structopt(
        short = "s",
        default_value = "133337",
        parse(try_from_str = "parse_seed")
    )]
    metaseed: u32,
}

impl Options {
    pub fn into_game_batch(self) -> GameBatch<RangeOpt, RangeOpt> {
        let Options {
            count_per_spec,
            dims_range,
            mines_range,
            metaseed
        } = self;

        GameBatch {
            count_per_spec,
            dims_range,
            mines_range,
            autoclear: true,
            metaseed,
            server_options: false
        }
    }
}