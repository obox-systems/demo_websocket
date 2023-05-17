#![warn(rust_2018_idioms)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]

//! This demo serves as a websocket server that supports high amounts of concurrent
//! connections and allows clients broadcast messages to every other client connected.

use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use futures::{
  channel::mpsc::{unbounded, UnboundedSender},
  future, SinkExt, StreamExt, TryStreamExt,
};
use tokio::{net::TcpListener, sync::Mutex};
use tokio_tungstenite::{accept_async, tungstenite::Message};

type Tx = UnboundedSender<Message>;
type PeerMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;

#[tokio::main]
async fn main() {
  let bind_addr = "127.0.0.1:9090";
  let listener = TcpListener::bind(bind_addr)
    .await
    .expect(format!("Failed to bind the websocket at {}", bind_addr).as_str());
  println!("Listening at {}", bind_addr);

  let peer_map = PeerMap::new(Mutex::new(HashMap::new()));
  while let Ok((stream, addr)) = listener.accept().await {
    let peer_map = peer_map.clone();
    tokio::spawn(async move {
      if let Ok(ws) = accept_async(stream).await {
        let (tx, rx) = unbounded();
        let addr = addr;
        println!("Client connected: {addr}");
        peer_map.lock().await.insert(addr, tx);

        let (outgoing, incoming) = ws.split();

        let peer_map_inner = peer_map.clone();
        let broadcast_future = incoming.try_for_each_concurrent(None, |msg| async {
          let msg = msg;
          println!("Received message\n```\n{msg}```\nFrom {addr}");
          let peers = peer_map_inner.lock().await;

          let clients = peers
            .iter()
            .filter(|(peer_addr, _)| peer_addr != &&addr)
            .map(|(_, ws_sink)| ws_sink);
          for mut client in clients {
            client.send(msg.clone()).await.unwrap();
          }

          Ok(())
        });

        let receive_future = rx.map(Ok).forward(outgoing);

        future::select(broadcast_future, receive_future).await;

        println!("Client disconnected: {addr}");
        peer_map.lock().await.remove(&addr);
      }
    });
  }
}
