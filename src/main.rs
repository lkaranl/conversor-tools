mod api;
mod compressor;
mod state;
mod utils;

use axum::Router;
use std::net::SocketAddr;
use std::time::Duration;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    let app_state = state::new_state();

    // Tarefa em background (Garbage Collector) para limpar a pasta uploads/
    tokio::spawn(async {
        let mut interval = tokio::time::interval(Duration::from_secs(600)); // Roda a cada 10 minutos
        interval.tick().await; // Consome o tick imediato do boot para não rodar assim que liga
        
        loop {
            interval.tick().await;
            println!("[Garbage Collector] Rodando limpeza de arquivos antigos na pasta 'uploads'...");
            
            if let Ok(mut entries) = tokio::fs::read_dir("uploads").await {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    if let Ok(metadata) = entry.metadata().await {
                        // Verifica se é arquivo e quando foi modificado
                        if metadata.is_file() {
                            if let Ok(modified) = metadata.modified() {
                                if let Ok(elapsed) = modified.elapsed() {
                                    // Deleta arquivos com mais de 10 minutos de vida (600 segundos)
                                    if elapsed > Duration::from_secs(600) {
                                        if let Err(e) = tokio::fs::remove_file(entry.path()).await {
                                            println!("[Garbage Collector] Erro ao remover {:?}: {}", entry.path(), e);
                                        } else {
                                            println!("[Garbage Collector] Arquivo expirado e removido: {:?}", entry.path());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    // Configuração de CORS para permitir requisições de qualquer origem
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .nest("/api", api::router(app_state))
        .fallback_service(ServeDir::new("static"))
        .layer(cors);

    // Alterado para 0.0.0.0 para aceitar conexões de qualquer IP na rede
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Servidor rodando em http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
