use futures::{SinkExt, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::connection::Connection;

mod io;
mod error;
mod connection;
mod data;

#[tokio::main]
async fn main() {
    // start connection
    let mut connection = Connection::new();
    connection.connect().await.unwrap();
    // listen for an interruption
    tokio::signal::ctrl_c().await.unwrap();
}

