#![feature(iterator_step_by)]
#![feature(inclusive_range_syntax)]
#![feature(range_contains)]
#![feature(conservative_impl_trait)]
#![feature(vec_resize_default)]

#![feature(proc_macro, conservative_impl_trait, generators)]

#[macro_use]
extern crate serde_derive;
extern crate itertools;
extern crate mersenne_twister;
extern crate rand;

extern crate serde_json;
extern crate tokio_core;

mod server;
mod coords;
mod game_grid;
mod client;
mod util;

// use self::tokio_core::reactor;
use std::error::Error;
use client::Client;
// use server::JsonServerWrapper
use server::NativeServer;

fn main() {
    main_try().unwrap();
}

fn main_try() -> Result<(), Box<Error>> {
    // let mut event_loop_core = reactor::Core::new()?;
    // let server = JsonServerWrapper::new_game(
    //     vec![25, 25],
    //     150,
    //     Some(109746378),
    //     true,
    //     &mut event_loop_core
    // )?;
    let server = NativeServer::new(vec![25, 25], 150, Some(109746378), true);
    let mut client = Client::new(server);

    let win = client.play()?;

    println!("{}", if win { "win!" } else { "lose!" });

    Ok(())
}