extern crate sigchain_client;
use sigchain_client::*;
use sigchain_client::block_validation::*;

extern crate serde_json;

fn main() {
    dotenv().ok();
    let _ = env_logger::init();
    println!("{}", serde_json::to_string_pretty(&gather_data()).unwrap());
}
