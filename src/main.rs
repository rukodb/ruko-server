//! Ruko server.
use ruko_server::server;

use tokio::net::TcpListener;
//use tokio::signal;

const PORT: u32 = 13377;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let listener = TcpListener::bind(&format!("127.0.0.1:{}", PORT)).await?;
    //server::run(listener, signal::ctrl_c()).await
    server::run(listener).await
}
