pub mod req {
    use coords::Coords;

    pub trait JsonServerRequest {
        const ACTION: &'static str;
    }

    #[derive(Serialize, Deserialize)]
    pub struct TurnRequest<'a> {
        pub id: &'a str,
        pub client: &'a str,
        pub clear: Vec<Coords>,
        pub flag: Vec<Coords>,
        pub unflag: Vec<Coords>,
    }

    impl<'a> JsonServerRequest for TurnRequest<'a> {
        const ACTION: &'static str = "turn";
    }

    #[derive(Serialize, Deserialize)]
    pub struct NewGameRequest<'a> {
        pub client: &'a str,
        pub seed: Option<u64>,
        pub dims: Vec<usize>,
        pub mines: usize,
        pub autoclear: bool,
    }

    impl<'a> JsonServerRequest for NewGameRequest<'a> {
        const ACTION: &'static str = "new";
    }

    #[derive(Serialize, Deserialize)]
    pub struct StatusRequest<'a> {
        pub id: &'a str
    }

    impl<'a> JsonServerRequest for StatusRequest<'a> {
        const ACTION: &'static str = "status";
    }
}

pub mod resp {
    extern crate chrono;

    use self::chrono::{DateTime, Utc};
    use coords::Coords;

    #[derive(Debug, Serialize, Deserialize, Clone, Copy)]
    #[serde(rename_all = "camelCase")]
    pub enum CellState { Cleared, Mine }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CellInfo {
        pub surrounding: usize,
        pub state: CellState,
        pub coords: Coords
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ServerResponse {
        pub id: String,
        pub seed: u64,
        pub dims: Vec<usize>,
        pub mines: usize,
        pub turn_num: usize,
        pub game_over: bool,
        pub win: bool,
        pub cells_rem: usize,
        pub flagged: Vec<Coords>,
        pub unflagged: Vec<Coords>,
        pub clear_actual: Vec<CellInfo>,
        pub clear_req: Vec<Coords>,
        pub turn_taken_at: DateTime<Utc>,
    }
}