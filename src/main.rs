use core::str::FromStr;
use std::{env, net::SocketAddr};

use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use session::handler;

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
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn index() -> impl IntoResponse {
    Html("<p>connect <strong>/session/:id</strong> to start/join a session </p>")
}
