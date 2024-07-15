use futures_util::{SinkExt, StreamExt};
use std::io::{self, Write};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;
use tokio::sync::mpsc;
use tokio::task;

pub async fn socket() {
    let url = Url::parse("ws://127.0.0.1:3000/ws").unwrap();
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");

    println!("Connected to the server");

    let (mut write, mut read) = ws_stream.split();
    let (tx, mut rx) = mpsc::channel::<String>(32);

    // Task zum Lesen von Nachrichten vom Server
    task::spawn(async move {
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    println!("Received: {}", text);
                }
                Ok(_) => {}
                Err(e) => {
                    println!("Error: {}", e);
                    break;
                }
            }
        }
    });

    // Task zum Senden von Nachrichten an den Server
    task::spawn(async move {
        while let Some(text) = rx.recv().await {
            if let Err(e) = write.send(Message::Text(text)).await {
                eprintln!("Error sending message: {}", e);
                break;
            }
        }
    });

    // Erfasse Benutzereingaben und sende sie an den Server
    loop {
        print!("Type a message: ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim().to_string();

        if input.is_empty() {
            continue;
        }

        if tx.send(input).await.is_err() {
            println!("Failed to send message");
            break;
        }
    }
}
