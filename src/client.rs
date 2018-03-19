extern crate tokio_core;

use std::error::Error;
use server_wrapper::JsonServerWrapper;
use game_grid::{GameGrid, Coords};
use self::tokio_core::reactor;

pub fn play(
    dims: Coords,
    mines: usize,
    seed: Option<u64>,
    event_loop_core: &mut reactor::Core
) -> Result<bool, Box<Error>> {
    let mut server_wrapper = JsonServerWrapper::new_game(
        dims.clone(),
        mines,
        seed,
        event_loop_core
    )?;
    let mut grid = GameGrid::new(dims.clone());
    let mut to_clear: Vec<Coords> = vec![
        dims.iter().map(|d| d / 2).collect()
    ];
    let mut to_flag = vec![];

    println!("{:?}", server_wrapper.status());

    while !(to_clear.is_empty() && to_flag.is_empty()) {
        println!("Turn {}", server_wrapper.status().turn_num);
        println!("to_clear {:?}", to_clear);
        println!("to_flag {:?}", to_flag);

        server_wrapper = server_wrapper.turn(
            to_clear,
            to_flag,
            vec![],
            event_loop_core
        )?;

        if server_wrapper.status().game_over { break; }

        let (grid_, next_actions) = grid.next_turn(
            &server_wrapper.status().clear_actual
        );

        grid = grid_;
        to_clear = next_actions.to_clear;
        to_flag = next_actions.to_flag;
    }

    Ok(server_wrapper.status().win)
}