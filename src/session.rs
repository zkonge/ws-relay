use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use dashmap::DashMap;
use futures::{
    future::join,
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use once_cell::sync::Lazy;
use tokio::sync::oneshot::{channel, Receiver, Sender};

static SESSIONS: Lazy<DashMap<String, Sender<WebSocket>>> = Lazy::new(DashMap::new);

pub async fn session_handler(wsu: WebSocketUpgrade, Path(id): Path<String>) -> impl IntoResponse {
    match SESSIONS.remove(&id) {
        // if session exists, act as responder
        Some((_, session)) => wsu
            .on_upgrade(|ws| async move {
                session.send(ws).unwrap();
            })
            .into_response(),

        // if session non-exists, act as initiator
        None => wsu
            .on_upgrade(|ws| async move {
                let (tx, rx) = channel::<WebSocket>();
                SESSIONS.insert(id.to_owned(), tx);
                bridge(ws, rx).await;
            })
            .into_response(),
    }
}

async fn bridge(initiator: WebSocket, responder_channel: Receiver<WebSocket>) {
    let responder = responder_channel.await.unwrap();
    let (i_tx, i_rx) = initiator.split();
    let (r_tx, r_rx) = responder.split();

    async fn relay(mut tx: SplitSink<WebSocket, Message>, mut rx: SplitStream<WebSocket>) {
        while let Some(Ok(m)) = rx.next().await {
            if tx.send(m).await.is_err() {
                break;
            }
        }
    }

    join(relay(r_tx, i_rx), relay(i_tx, r_rx)).await;
}
