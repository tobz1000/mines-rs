#[macro_use]
extern crate serde_derive;
extern crate tokio_core;

mod server_wrapper;

use self::tokio_core::reactor;
use server_wrapper::{JsonServerWrapper};

fn main() {
	let mut event_loop_core = reactor::Core::new().unwrap();
	let server = JsonServerWrapper::new_game(
		vec![10, 10], 10, None, &mut event_loop_core
	).unwrap();
	println!("{}", server.status().id);
}