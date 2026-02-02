mod utils;
use kademlia::{startup::start_server};
use env_logger::Env;




#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    start_server().await;


    tokio::signal::ctrl_c().await.unwrap();

    Ok(())
}
