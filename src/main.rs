use std::{collections::HashMap, env, io::Error as IOError, net::SocketAddr, sync::Arc};

use futures::{SinkExt, StreamExt};
use log::info;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{
        mpsc::{unbounded_channel, UnboundedSender},
        Mutex,
    },
};

const INDEX_HMTL: &str = "<html>
hello World
</html>";

use tokio_tungstenite::tungstenite::Message;

type State = Arc<Mutex<HashMap<SocketAddr, UnboundedSender<String>>>>;

pub fn state() -> State {
    Arc::new(Mutex::new(HashMap::new()))
}

#[tokio::main]
async fn main() -> Result<(), IOError> {
    env_logger::init();
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "0.0.0.0:8080".to_string());
    let socket = TcpListener::bind(&addr).await?;
    info!("start server to listen addr : {}", &addr);
    let state = state();
    while let Ok((stream, addr)) = socket.accept().await {
        tokio::spawn(handler(stream, addr, state.clone()));
    }
    Ok(())
}

async fn handler(stream: TcpStream, addr: SocketAddr, state: State) -> Result<(), IOError> {
    info!("client addr : {} connect to server", addr);
    let (mut outgoing, mut incoming) = tokio_tungstenite::accept_async(stream)
        .await
        .map_err(|e| IOError::new(std::io::ErrorKind::Other, e))?
        .split();
    let (tx, mut rx) = unbounded_channel::<String>();
    {
        let mut guard = state.lock().await;
        guard.insert(addr, tx);
    }
    outgoing.send(Message::Text(INDEX_HMTL.to_string())).await;
    broadcast(&state, &addr, &addr.to_string()).await;
    loop {
        tokio::select! {
            Some(msg) = rx.recv() => {
                outgoing.send(Message::Text(msg)).await;
            }
            Some(msg) = incoming.next() => {
                let msg = msg.unwrap();
                let msg = msg.to_text().unwrap();
                broadcast(&state, &addr, msg).await;
            }
        }
    }
}

pub async fn broadcast(state: &State, sender_addr: &SocketAddr, msg: &str) {
    let guard = state.lock().await;
    for (addr, tx) in guard.iter() {
        if addr != sender_addr {
            tx.send(msg.to_string());
        }
    }
}
