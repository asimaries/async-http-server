mod consts;

use std::{error::Error, time::Duration};

use consts::{GET, SLEEP};
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

async fn handle_client(mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).await.expect("Failed to read");

    let request = String::from_utf8_lossy(&buffer[..]);
    println!("LOG: {}", request);

    let (status_line, filename) = if buffer.starts_with(GET) {
        ("HTTP/1.1 200 OK\r\n\r\n", "static/index.html")
    } else if buffer.starts_with(SLEEP) {
        tokio::time::sleep(Duration::from_secs(5)).await;
        ("HTTP/1.1 200 OK\r\n\r\n", "static/index.html")
    } else {
        ("HTTP/1.1 200 OK\r\n\r\n", "static/404.html")
    };
    let content = fs::read_to_string(filename).await.unwrap_or_default();

    let response = format!("{status_line}{content}");
    stream
        .write_all(response.as_bytes())
        .await
        .expect("Failed to write response");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let listener = TcpListener::bind("127.0.0.1:8000").await?;
    println!("Server listening on http://localhost:8000");
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                tokio::spawn(async move {
                    if let Err(e) = handle_client(stream).await {
                        eprintln!("Error handling client: {}", e);
                    }
                });
            }
            Err(err) => {
                println!("{:?}", err);
            }
        }
    }
}
