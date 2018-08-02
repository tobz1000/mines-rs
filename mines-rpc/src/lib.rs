extern crate grpcio;
extern crate protobuf;
extern crate futures;

mod protogen;

use std::sync::Arc;
use grpcio::{Environment, RpcContext, UnarySink, Server, ServerBuilder};
use protogen::mines_grpc::{Mines, create_mines};
use protogen::mines::{Request, Response};

#[derive(Clone)]
struct MinesService;

impl Mines for MinesService {
    fn batch(&self, ctx: RpcContext, req: Request, sink: UnarySink<Response>){
        println!("{:?}", req);
        sink.success(Response::new());
    }
}

pub fn grpc_server(port: u16) -> Result<Server, grpcio::Error> {
    let env = Arc::new(Environment::new(1));
    let service = create_mines(MinesService);

    ServerBuilder::new(env)
        .register_service(service)
        .bind("127.0.0.1", port)
        .build()
}