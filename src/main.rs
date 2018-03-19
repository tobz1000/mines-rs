#![feature(iterator_step_by)]
#![feature(inclusive_range_syntax)]
#![feature(range_contains)]
#![feature(conservative_impl_trait)]
#![feature(try_from)]

#![feature(proc_macro, conservative_impl_trait, generators)]

#[macro_use]
extern crate serde_derive;
extern crate itertools;

extern crate serde_json;
extern crate tokio_core;

mod server_wrapper;
mod game_grid;
mod cell;
mod client;

use self::tokio_core::reactor;
use std::error::Error;

fn main() {
	main_try().unwrap();
}

fn main_try() -> Result<(), Box<Error>> {
	let mut event_loop_core = reactor::Core::new()?;
	let win = client::play(
		vec![25, 25],
		100,
		None,
		&mut event_loop_core
	)?;

	println!("{}", if win { "win!" } else { "lose!" });

	Ok(())
}