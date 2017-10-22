#[macro_use]
extern crate serde_derive;

mod server_wrapper;

use server_wrapper::{JsonServerWrapper,MinesServer};

fn main() {
	let server = JsonServerWrapper::new_game(vec![10, 10], 10, None).unwrap();
	println!("{}", server.status().id);
}