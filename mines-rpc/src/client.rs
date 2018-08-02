
use std::sync::Arc;
use grpcio::{Environment, ChannelBuilder};
use protogen::mines_grpc::MinesClient;

pub fn grpc_client(port: u16) -> MinesClient {
    let env = Arc::new(Environment::new(1));
    let channel = ChannelBuilder::new(env)
        .connect(format!("localhost:{}", port).as_str());

    MinesClient::new(channel)
}