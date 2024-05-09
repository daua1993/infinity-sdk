use futures::{SinkExt, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::connection::Connection;

mod io;
mod error;
mod connection;
mod data;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // start connection
    let mut connection = Connection::new();
    let (tx, mut rx) = tokio::sync::mpsc::channel(100);
    connection.expect("REASON").connect(tx).await.unwrap();
    let dt = rx.recv().await.unwrap();
    println!("Received data: {:?}", dt);
    // listen for an interruption
    tokio::signal::ctrl_c().await.unwrap();
    Ok(())
}

