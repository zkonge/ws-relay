use core::str::FromStr;
use std::{env, net::SocketAddr};

use axum::{
    Router,
    response::{Html, IntoResponse},
    routing::get,
};
use session::handler;
use tokio::net::TcpListener;

mod session;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:3000".to_owned());

    let app = Router::new()
        .route("/", get(index))
        .route("/session/:id", get(handler));

    let addr = SocketAddr::from_str(&addr).unwrap();

    println!("Listen on {addr}");
    axum::serve(TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}

async fn index() -> impl IntoResponse {
    Html("<p>connect <strong>/session/:id</strong> to start/join a session </p>")
}
