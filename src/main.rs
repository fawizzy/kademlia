use env_logger::Env;
use kademlia::network::server::start_server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    start_server().await;

    tokio::signal::ctrl_c().await.unwrap();

    Ok(())
}
