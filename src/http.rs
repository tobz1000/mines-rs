extern crate rocket;

use game_batch::GameBatch;
use server::{NativeServer, NativeServerConfig};

#[post("/batch", data = "<batch_spec>")]
pub fn run_batch(batch_spec: GameBatch) {
    batch.run::<NativeServer>(NativeServerConfig { save_to_db: false })
}