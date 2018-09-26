#![recursion_limit="128"]

#[macro_use] extern crate yew;
#[macro_use] extern crate stdweb;
extern crate mines_rs;

use std::iter::once;
use yew::prelude::{
    Component,
    ComponentLink,
    Html,
    Renderable,
    ShouldRender,
    App
};
use yew::virtual_dom::VText;
use mines_rs::{NativeServer, GameBatch, SpecResult};

struct GameViewer {
    // turns:
}

struct GameViewerProps {

}

impl Component for GameViewer {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, _comp: ComponentLink<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }
}

impl Renderable<GameViewer> for GameViewer {
    fn view(&self) -> Html<Self> {
        // let GameViewerProps
        html! {
            <div class="gameArea",>
                // <GameGrid turn_info=, />
            </div>
        }
    }
}

fn main() {
    yew::initialize();
    App::<GameViewer>::new().mount_to_body();

    run_sample();

    yew::run_loop();
}

fn run_sample() {
    let count_per_spec = 100;
    let batch = mines_rs::GameBatch {
        count_per_spec,
        dims_range: vec![once(20), once(20)],
        mines_range: (10..=50).step_by(5),
        autoclear: true,
        metaseed: 133337
    };

    let start = js! { return performance.now(); };

    let results = batch.run(
        |spec| mines_rs::NativeServer::new(spec, false),
        |_game| ()
    ).unwrap();

    let end = js! { return performance.now(); };
    let game_count = (results.len() * count_per_spec) as i32;

    js! {
        console.table(@{results});
        const durMs = (@{end} - @{start});
        const durS = durMs / 1000;
        const avgUs = (durMs * 1000) / @{game_count};
        console.log("Time: " + durMs.toFixed(0) / 1000 + "s (avg " + avgUs.toFixed(0) + "µs/game)");
    }
}