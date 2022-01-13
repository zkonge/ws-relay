use std::time::Duration;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, WebSocketUpgrade,
    },
    http::StatusCode,
    response::IntoResponse,
};
use dashmap::DashMap;
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use once_cell::sync::Lazy;
use tokio::{
    select,
    sync::oneshot::{channel, Sender},
    time::timeout,
};

static SESSIONS: Lazy<DashMap<String, Sender<WebSocket>>> = Lazy::new(DashMap::new);

pub async fn handler(wsu: WebSocketUpgrade, Path(id): Path<String>) -> impl IntoResponse {
    if id.len() > 32 {
        return (StatusCode::FORBIDDEN, "Max session id length is 32").into_response();
    }

    let session = SESSIONS.remove(&id);

    match session {
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

                SESSIONS.insert(id.clone(), tx);

                match timeout(Duration::from_secs(180), rx).await {
                    Ok(responder) => {
                        bridge(ws, responder.unwrap()).await;
                    }
                    Err(_) => {
                        SESSIONS.remove(&id);
                    }
                };
            })
            .into_response(),
    }
}

async fn bridge(initiator: WebSocket, responder: WebSocket) {
    async fn relay(tx: &mut SplitSink<WebSocket, Message>, rx: &mut SplitStream<WebSocket>) {
        while let Some(Ok(m)) = rx.next().await {
            if tx.send(m).await.is_err() {
                break;
            }
        }
    }

    let (mut i_tx, mut i_rx) = initiator.split();
    let (mut r_tx, mut r_rx) = responder.split();

    let _ = select! {
        _ = relay(&mut r_tx,&mut i_rx) => i_tx.send(Message::Close(None)).await,
        _ = relay(&mut i_tx,&mut r_rx) => r_tx.send(Message::Close(None)).await
    };
}
