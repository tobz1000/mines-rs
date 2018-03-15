extern crate tokio_core;

use std::error::Error;
use server_wrapper::JsonServerWrapper;
use game_grid::GameGrid;
use self::tokio_core::reactor;

pub fn play(
    dims: Vec<usize>,
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
    let mut to_clear: Vec<Vec<usize>> = vec![
        dims.iter().map(|d| d / 2).collect()
    ];
    let mut to_flag = vec![];

    println!("{:?}", server_wrapper.status());

    while !(to_clear.is_empty() && to_flag.is_empty()) {
        server_wrapper = server_wrapper.turn(
            to_clear,
            to_flag,
            vec![],
            event_loop_core
        )?;

        if server_wrapper.status().game_over { break; }

        let next_actions = grid.handle_cell_info(
            server_wrapper.status().clear_actual.as_slice()
        );

        to_clear = next_actions.to_clear;
        to_flag = next_actions.to_flag;
    }

    Ok(server_wrapper.status().win)
}