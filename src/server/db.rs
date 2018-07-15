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
    pub surrounding: i32,
    pub state: CellState,
    pub coords: Coords<i32>
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub created_at: DateTime<Utc>,
    pub pass: Option<String>,
    pub seed: i32,
    pub dims: Vec<i32>,
    pub size: i32,
    pub mines: i32,
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
    pub clear_req: Vec<Coords<i32>>,
    pub clear_actual: Vec<CellInfo>,
    pub flagged: Vec<Coords<i32>>,
    pub unflagged: Vec<Coords<i32>>,
    pub game_over: bool,
    pub win: bool,
    pub cells_rem: i32,
}

impl<'a> Model<'a> for Game {
    const COLLECTION_NAME: &'static str = "games";

    fn id(&self) -> Option<ObjectId> {
        self.id.clone()
    }

    fn set_id(&mut self, oid: ObjectId) {
        self.id = Some(oid);
    }
}