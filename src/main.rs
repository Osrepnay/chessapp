use futures::stream::SplitSink;
use futures::{SinkExt, StreamExt};
use rand;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::filters::ws::Message;
use warp::ws::{WebSocket, Ws};
use warp::Filter;

#[tokio::main]
async fn main() {
    let websockets: Arc<RwLock<HashMap<u32, HashMap<usize, SplitSink<WebSocket, Message>>>>> =
        Arc::new(RwLock::new(HashMap::new()));
    let websockets = warp::any().map(move || websockets.clone());

    let index = warp::get().and(warp::fs::dir("static"));

    let move_ws =
        warp::path("movews").and(warp::ws()).and(websockets).map(
            |ws: Ws,
             websockets: Arc<
                RwLock<HashMap<u32, HashMap<usize, SplitSink<WebSocket, Message>>>>,
            >| {
                ws.on_upgrade(|websocket| async move {
                    let (mut tx, mut rx) = websocket.split();
                    let websockets_lock = websockets.clone();

                    let game_id = rx.next().await.unwrap().unwrap();
                    let game_id = if game_id.is_text() {
                        game_id.to_str().unwrap().parse().unwrap()
                    } else {
                        return;
                    };
                    eprintln!("new client connected with game id {}", game_id);
                    let websockets_read = websockets_lock.read().await;
                    // is first websocket with id
                    let is_first_websocket = !websockets_read.contains_key(&game_id);
                    let key = if is_first_websocket {
                        rand::random()
                    } else {
                        let sinks = websockets_read.get(&game_id).unwrap(); // guaranteed to be Some
                        let mut potential_key = rand::random();
                        while sinks.contains_key(&potential_key) {
                            potential_key = rand::random();
                        }
                        potential_key
                    };
                    drop(websockets_read);
                    tx.send(Message::text(if is_first_websocket {
                        "white"
                    } else {
                        "black"
                    }))
                    .await
                    .unwrap();
                    let mut websockets_write = websockets_lock.write().await;
                    if is_first_websocket {
                        let mut new_hashmap = HashMap::new();
                        new_hashmap.insert(key, tx);
                        (*websockets_write).insert(game_id, new_hashmap);
                    } else {
                        let sinks = websockets_write.get_mut(&game_id).unwrap();
                        sinks.insert(key, tx);
                    }
                    drop(websockets_write);
                    eprintln!("finished connecting client key {} game id {}", key, game_id);
                    loop {
                        let move_msg = rx.next().await;
                        eprintln!(
                            "recieved message: {:?} from client key {} and game id {}",
                            move_msg, key, game_id
                        );
                        let mut websockets_write = websockets_lock.write().await;
                        if let Some(Ok(move_msg)) = move_msg {
                            if move_msg.is_text() {
                                let move_msg = move_msg.to_str().unwrap(); //guaranteed to be str
                                for (curr_key, tx) in
                                    (*websockets_write.get_mut(&game_id).unwrap()).iter_mut()
                                {
                                    if *curr_key != key {
                                        tx.send(Message::text(move_msg)).await.unwrap()
                                    }
                                }
                            }
                        } else {
                            eprintln!("client key {} disconnected from game id {}", key, game_id);
                            let sinks = websockets_write.get_mut(&game_id).unwrap();
                            // remove whole key if no more sinks left
                            if sinks.len() > 1 {
                                sinks.remove(&key);
                            } else {
                                drop(sinks);
                                websockets_write.remove(&game_id);
                            }
                            break;
                        }
                    }
                })
            },
        );
    let routes = index.or(move_ws);
    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
}
