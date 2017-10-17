#[macro_use]
extern crate serde_derive;

mod server_wrapper;

#[derive(Serialize, Deserialize, Debug)]
struct ServerResponse {
	id: String,
}

fn main() {
}