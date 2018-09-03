use server::native::NativeServer;
use ::GameError;

mod mongodb_connector;

pub trait DbInserter: Sync {
    fn insert_game(&self, server: &NativeServer) -> Result<(), GameError>;
}

pub struct MongoDbInserter;

impl DbInserter for MongoDbInserter {
    fn insert_game(&self, server: &NativeServer) -> Result<(), GameError> {
        Ok(mongodb_connector::insert_game(server)?)
    }
}

pub struct MemDbInserter;

impl DbInserter for MemDbInserter {
    fn insert_game(&self, _server: &NativeServer) -> Result<(), GameError> {
        panic!("not impl'd");
    }
}