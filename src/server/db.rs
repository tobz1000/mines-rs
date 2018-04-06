extern crate bson;
extern crate chrono;
extern crate serde;
extern crate wither;

use self::chrono::{DateTime, Utc};
use self::bson::oid::ObjectId;
use self::wither::Model;
use coords::Coords;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum CellState { Empty, Cleared, Mine }

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CellInfo {
    pub surrounding: usize,
    pub state: CellState,
    pub coords: Coords
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub created_at: DateTime<Utc>,
    pub pass: Option<String>,
    pub seed: u32,
    pub dims: Vec<usize>,
    pub size: usize,
    pub mines: usize,
    pub autoclear: bool,
    pub turns: Vec<Turn>,
    pub clients: Vec<String>,
    pub cell_array: Vec<CellState>,
    pub flag_array: Vec<bool>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Turn {
    pub turn_taken_at: DateTime<Utc>,
    pub clear_req: Vec<Coords>,
    pub clear_actual: Vec<CellInfo>,
    pub flagged: Vec<Coords>,
    pub unflagged: Vec<Coords>,
    pub game_over: bool,
    pub win: bool,
    pub cells_rem: usize,
}

impl<'a> Model<'a> for Game {
    const COLLECTION_NAME: &'static str = "games";

    fn id(&self) -> Option<ObjectId> {
        return self.id.clone();
    }

    fn set_id(&mut self, oid: ObjectId) {
        self.id = Some(oid);
    }
}