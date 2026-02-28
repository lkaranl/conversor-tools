mod api;
mod compressor;
mod state;

use axum::Router;
use std::net::SocketAddr;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    let app_state = state::new_state();

    let app = Router::new()
        .nest("/api", api::router(app_state))
        .fallback_service(ServeDir::new("static"));

    // Alterado para 0.0.0.0 para aceitar conexões de qualquer IP na rede
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Servidor rodando em http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
