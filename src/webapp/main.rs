#![recursion_limit="128"]

mod game_runner;

use std::iter::once;
use stdweb::{js, _js_impl};
use yew::{html, html_impl};
use yew::prelude::{
    Component,
    ComponentLink,
    Html,
    Renderable,
    ShouldRender,
    App
};
use yew::agent::{
    Bridge,
    Bridged
};
use mines_rs::{NativeServer, GameBatch, SpecResult};
use game_runner::{GameBatchRunner, GameBatchMessage, GameBatchResultMessage};

enum GameViewerMsg {
    DoBatch,
    BatchResult(Vec<SpecResult<()>>)
}

struct GameViewer {
    game_runner: Box<dyn Bridge<GameBatchRunner>>
}

impl GameViewer {
    fn do_batch(&mut self) {
        let count_per_spec = 100;
        let batch = GameBatch {
            count_per_spec,
            dims_range: vec![once(20), once(20)],
            mines_range: (10..=50).step_by(5),
            autoclear: true,
            metaseed: 133337
        }.into_serializable();

        self.game_runner.send(GameBatchMessage(batch));
    }
}

impl Component for GameViewer {
    type Message = GameViewerMsg;
    type Properties = ();

    fn create(_props: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        let callback = link.send_back(|GameBatchResultMessage(results)| {
            GameViewerMsg::BatchResult(results)
        });
        let game_runner = GameBatchRunner::bridge(callback);

        Self { game_runner }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            GameViewerMsg::DoBatch => { self.do_batch(); },
            GameViewerMsg::BatchResult(results) => {
                js! { console.table(@{results}); }
            }
        }

        false
    }
}

impl Renderable<GameViewer> for GameViewer {
    fn view(&self) -> Html<Self> {
        html! {
            <div class="gameArea",>
                <button onclick=|_| GameViewerMsg::DoBatch,>{"clicky"}</button>
            </div>
        }
    }
}

fn main() {
    yew::initialize();
    App::<GameViewer>::new().mount_to_body();
    yew::run_loop();
}

fn run_sample() {
    let count_per_spec = 100;
    let batch = GameBatch {
        count_per_spec,
        dims_range: vec![once(20), once(20)],
        mines_range: (10..=50).step_by(5),
        autoclear: true,
        metaseed: 133337
    }.into_serializable();

    let start = js! { return performance.now(); };

    let results = batch.run(
        |spec| NativeServer::new(spec, false),
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