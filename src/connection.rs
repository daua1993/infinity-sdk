use std::sync::Arc;
use std::time::Duration;

use futures::{SinkExt, StreamExt};
use serde_json::json;
use tokio::io::WriteHalf;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_util::codec::{FramedRead, FramedWrite};

use crate::data::Data;
use crate::io::AppCodec;

pub struct Connection {
    pub running: bool,
    pub transport_tx: Option<Arc<Mutex<FramedWrite<WriteHalf<TcpStream>, AppCodec>>>>,
}

impl Connection {
    pub fn new() -> Self {
        Connection {
            running: true,
            transport_tx: None,
        }
    }

    fn set_transport_tx(&mut self, tx: Arc<Mutex<FramedWrite<WriteHalf<TcpStream>, AppCodec>>>) {
        self.transport_tx = Some(tx);
    }

    async fn on_data_received(&mut self, data: Data) {
        println!("Nhan duoc du lieu tai day: {:?}", data);
    }

    pub async fn connect(&mut self) -> anyhow::Result<()> {
        loop {
            if !self.running {
                break;
            }
            // let mut receiver = receiver_from_worker.resubscribe();
            // let mut sender = sender_to_worker.clone();
            match TcpStream::connect("localhost:9999").await {
                Ok(mut stream) => {
                    println!("Connected to the server");

                    let esl_codec = AppCodec {};
                    let (read_half, write_half) = tokio::io::split(stream);

                    let mut transport_rx = FramedRead::new(read_half, esl_codec.clone());
                    let transport_tx = Arc::new(Mutex::new(FramedWrite::new(write_half, esl_codec.clone())));
                    // let transport_tx = FramedWrite::new(write_half, esl_codec.clone());
                    self.set_transport_tx(transport_tx.clone());

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
                        if let (Some(Ok(dt))) = transport_rx.next().await {
                            println!("Received from server: {:?}", dt);
                            self.on_data_received(dt.clone()).await;
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
                                    Ok(_) => { println!("Sent to the server") }
                                    Err(err) => { println!("Error sending to the server: {:?}", err) }
                                };
                                drop(tx);
                            }
                        } else {
                            println!("Error reading from the server");
                            break;
                        }
                        tokio::time::sleep(Duration::from_millis(5)).await;
                    }
                }
                Err(err) => {
                    println!("Error connecting to the server: {:?}", err);
                }
            }
            println!("Reconnecting in 5 secs");
            let secs_30 = Duration::from_secs(5);
            tokio::time::sleep(secs_30).await;
        }
        Ok(())
    }
}