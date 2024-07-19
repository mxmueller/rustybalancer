use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;
use tokio::sync::{mpsc, Mutex};
use tokio::task;

pub type WebSocketSender = Arc<Mutex<mpsc::Sender<String>>>;
pub type WebSocketReceiver = Arc<Mutex<mpsc::Receiver<Message>>>;

pub async fn connect_socket() -> (WebSocketSender, WebSocketReceiver) {
    let url = Url::parse("ws://deployment-agent:2547/ws").unwrap();
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");

    println!("Connected to the server");

    let (mut write, mut read) = ws_stream.split();
    let (tx, mut rx) = mpsc::channel::<String>(32);
    let (msg_tx, msg_rx) = mpsc::channel::<Message>(32);
    let msg_rx = Arc::new(Mutex::new(msg_rx));
    let tx = Arc::new(Mutex::new(tx));

    // Task for reading messages
    task::spawn({
        let msg_tx = msg_tx.clone();
        async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(msg) => {
                        if msg_tx.send(msg).await.is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        println!("Error: { }", e);
                        break;
                    }
                }
            }
        }
    });

    // Sending events:
    //send_event(&mut write).await;

    // Task for sending messages.
    task::spawn(async move {
        while let Some(text) = rx.recv().await {
            //let mut tx = tx.lock().await;
            if let Err(e) = write.send(Message::Text(text)).await {
                eprintln!("Error sending message: {}", e);
                break;
            }
        }
    });

    (tx, msg_rx)
}

async fn send_event() {
    // TODO! sending events
}

