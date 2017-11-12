#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate itertools;

extern crate serde_json;
extern crate tokio_core;

mod server_wrapper;
mod client;

use self::tokio_core::reactor;
use server_wrapper::{JsonServerWrapper};
use std::error::Error;

fn main() {
	main_try().unwrap();
}

fn main_try() -> Result<(), Box<Error>> {
	let mut event_loop_core = reactor::Core::new()?;
	let server = JsonServerWrapper::new_game(
		vec![10, 10], 10, None, &mut event_loop_core
	)?;
	println!("{}", serde_json::to_string(server.status())?);

	Ok(())
}