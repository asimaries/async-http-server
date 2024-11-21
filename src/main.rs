mod consts;

use std::{error::Error, time::Duration};

use consts::{GET, SLEEP};
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::mpsc,
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

// #[tokio::main]
fn main() -> Result<(), Box<dyn Error>> {
    let listener_runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .thread_name("acceptor-pool")
        .enable_all()
        .build()?;

    let handler_runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(8)
        .thread_name("handler-pool")
        .enable_all()
        .build()?;

    let (tx, mut rx) = mpsc::channel::<TcpStream>(4000);

    handler_runtime.spawn(async move {
        while let Some(sock) = rx.recv().await {
            tokio::spawn(async move {
                if let Err(e) = handle_client(sock).await {
                    eprintln!("Error handling client: {}", e);
                }
            });
        }
    });
    listener_runtime.block_on(async move {
        let listener = match TcpListener::bind("127.0.0.1:8000").await {
            Ok(listener) => listener,
            Err(err) => panic!("error binding tcp listener: {}", err),
        };
        println!("Server listening on http://localhost:8000");
        loop {
            let sock = match accept_conn(&listener).await {
                Ok(stream) => stream,

                Err(err) => {
                    println!("{:?}", err);
                    panic!("{:?}", err);
                }
            };
            let _ = tx.send(sock).await;
        }
    });

    Ok(())
}

async fn accept_conn(listener: &TcpListener) -> Result<TcpStream, Box<dyn Error>> {
    loop {
        match listener.accept().await {
            Ok((stream, _)) => return Ok(stream),
            Err(e) => panic!("error accepting connection: {}", e),
        }
    }
}
