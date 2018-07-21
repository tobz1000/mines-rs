#[macro_use] extern crate yew;

use yew::{Component, ComponentLink};

struct GameViewer {
    games: Vec<Game>,
    current_id: Option<String>
}

impl Component for Model {
    type Message = ();
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        GameViewer { games: vec![], current_id: None }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        true
    }
}

impl Renderable<Model> for Model {
    fn view(&self) -> Html<Self> {
        html! {
            // Render your model here
            // <button onclick=|_| yew::services::ConsoleService.log("aldskasa")>
            //     { "Click me!" }
            // </button>
        }
    }
}

fn main() {
    yew::initialize();
    App::<GameViewer>::new().mount_to_body();
    yew::run_loop();
}