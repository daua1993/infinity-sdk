mod io;
mod error;

use std::rc::Rc;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use byteorder::{BigEndian};
use serde_json::json;
use tokio::sync::broadcast::{Sender, Receiver, channel};
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{broadcast, Mutex};
use futures::SinkExt;
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, FramedWrite};
use crate::error::EslError;
use crate::io::EslCodec;

#[tokio::main]
async fn main() {
    // sender / receiver - for data received from the server
    let (from_tcp_sr, from_tcp_rr) = channel(100);

    // sender / receiver - for data to send to the server
    let (to_tcp_sr, to_tcp_rr) = channel(100);

    // spawn a worker thread
    let work = tokio::spawn(process(to_tcp_sr, from_tcp_rr));

    // spawn a connector
    let tcp = tokio::spawn(connect(from_tcp_sr, to_tcp_rr));

    // listen for an interruption
    tokio::signal::ctrl_c().await;
    // before exiting
    work.abort();
    tcp.abort();
}

#[derive(Debug, Clone)]
pub struct Data {
    service: u16,
    data: String,
}

impl Data {
    async fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        let len = self.data.len() as u32;
        buf.write_u32(len).await.unwrap();
        buf.write_u16(self.service).await.unwrap();
        buf.extend_from_slice(&self.data.to_string().as_bytes());
        buf
    }
}

async fn process(sender_to_tcp: Sender<Data>, receiver_from_tcp: Receiver<Data>) {

    // use tokio::select!

    // and either forward to TCP the results of your async work
    // or get data from TCP and do something with it

    // in a loop
}

async fn connect(sender_to_worker: Sender<Data>, mut receiver_from_worker: Receiver<Data>) {
    loop {
        let mut receiver = receiver_from_worker.resubscribe();
        let mut sender = sender_to_worker.clone();
        match TcpStream::connect("localhost:9999").await {
            Ok(mut stream) => {
                println!("Connected to the server");

                let esl_codec = EslCodec {};
                let (read_half, write_half) = tokio::io::split(stream);

                let mut transport_rx = FramedRead::new(read_half, esl_codec.clone());
                let transport_tx = Arc::new(Mutex::new(FramedWrite::new(write_half, esl_codec.clone())));
                // let transport_tx = FramedWrite::new(write_half, esl_codec.clone());

                // Creating a dynamic JSON object
                let user = json!({
                    "requestId": "1",
                    "username": "stringee",
                    "password": "abc@1234",
                    "module_name": "client",
                    "dc_id": "dc_id",
                    "ip": "127.0.0.1",
                    "port": "9999",
                });

                // Serialize the JSON object to a string
                let json_string = serde_json::to_string(&user).unwrap();

                let dt = Data { service: 2, data: json_string };
                let b1 = dt.to_bytes().await;
                let b2: &[u8] = &b1;

                let mut tx = transport_tx.lock().await;
                tx.send(b2).await.unwrap();
                drop(tx);

                loop {
                    let t = transport_rx.next().await;
                    match t {
                        None => { println!("None") }
                        Some(o) => {
                            match o {
                                Ok(dt) => {
                                    println!("Received from server: {:?}", dt);
                                    if dt.service == 1 {
                                        let ping = json!({
                                            "requestId": "1",
                                            "hbc": "1",
                                        });
                                        let str = serde_json::to_string(&ping).unwrap();
                                        let dt = Data { service: 1, data: str };
                                        let v1 = dt.to_bytes().await;
                                        let v2: &[u8] = &v1;
                                        let mut tx = transport_tx.lock().await;
                                        match tx.send(v2).await {
                                            Ok(_) => {println!("Sent to the server")}
                                            Err(err) => { println!("Error sending to the server: {:?}", err) }
                                        };
                                        drop(tx);
                                    }
                                }
                                Err(e) => {
                                    println!("Error reading from the server: {:?}", e);
                                }
                            }
                        }
                    }
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
            }
            Err(err) => {
                println!("Error connecting to the server: {:?}", err);
            }
        }
        println!("Reconnecting in 5 secs");
        let secs_30 = core::time::Duration::from_secs(5);
        tokio::time::sleep(secs_30).await;
    }
}