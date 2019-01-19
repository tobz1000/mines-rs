use mines_rs::{GameBatch, NativeServer, SpecResult};
use serde_derive::{Deserialize, Serialize};
use yew::agent::{Agent, AgentLink, HandlerId, Transferable};

mod agent_link_type {
    pub use yew::agent::{
        Context, // Runs on same thread
        Global,  // panics (unimplemented)
        Job,     // Runs on same thread
        Private, // Doesn't seem to run
        Public,  // Doesn't seem to run
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
        let results = batch
            .0
            .run(|spec| NativeServer::new(spec, false), |_game| ())
            .unwrap();

        self.link.response(who, GameBatchResultMessage(results));
    }
}
