use serde_derive::{Serialize, Deserialize};
use mines_rs::{GameBatch, SpecResult, NativeServer };
use yew::agent::{
    Agent,
    AgentLink,
    HandlerId,
    Transferable
};

mod agent_link_type {
    pub use yew::agent::{
        Job, // Runs on same thread
        Context, // Runs on same thread
        Public, // Doesn't seem to run
        Private, // Doesn't seem to run
        Global // panics (unimplemented)
    };
}

pub struct GameBatchRunner {
    link: AgentLink<GameBatchRunner>,
}

#[derive(Serialize, Deserialize)]
pub struct GameBatchMessage(pub GameBatch<Vec<usize>, Vec<usize>>);

#[derive(Serialize, Deserialize)]
pub struct GameBatchResultMessage(pub Vec<SpecResult<()>>);

impl Transferable for GameBatchMessage {}
impl Transferable for GameBatchResultMessage {}

impl Agent for GameBatchRunner {
    type Reach = agent_link_type::Job;
    type Message = ();
    type Input = GameBatchMessage;
    type Output = GameBatchResultMessage;

    fn create(link: AgentLink<Self>) -> Self {
        GameBatchRunner { link }
    }

    fn update(&mut self, msg: Self::Message) {}

    fn handle(&mut self, batch: Self::Input, who: HandlerId) {
        let results = batch.0.run(
            |spec| NativeServer::new(spec, false),
            |_game| ()
        ).unwrap();

        self.link.response(who, GameBatchResultMessage(results));
    }
}
