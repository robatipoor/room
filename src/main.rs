use std::{collections::HashMap, env, io::Error as IOError, net::SocketAddr, sync::Arc};

use futures::{channel::mpsc::unbounded, stream::SplitSink, StreamExt};
use log::info;
use tokio::{
    net::{TcpListener, TcpSocket, TcpStream},
    sync::{
        mpsc::{unbounded_channel, Sender},
        Mutex,
    },
};

type State = Arc<Mutex<HashMap<String, Sender<String>>>>;

#[tokio::main]
async fn main() -> Result<(), IOError> {
    env_logger::init();
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "0.0.0.0:8080".to_string());
    let socket = TcpListener::bind(&addr).await?;
    info!("start listen addr : {}", &addr);
    while let Ok((stream, addr)) = socket.accept().await {
        tokio::spawn(handler(stream, addr));
    }
    Ok(())
}

async fn handler(stream: TcpStream, addr: SocketAddr) -> Result<(), IOError> {
    info!("client addr : {} connect to server", addr);
    let (outgoing, mut incoming) = tokio_tungstenite::accept_async(stream)
        .await
        .map_err(|e| IOError::new(std::io::ErrorKind::Other, e))?
        .split();
    let (tx, rx) = unbounded::<String>();
    let msg = incoming.next().await.unwrap().unwrap();
    info!(">> {}", msg);
    rx.forward(sink)

    Ok(())
}
