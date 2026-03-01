use crate::state::AppState;
use crate::compressor::{compress_media, MediaType};
use axum::{
    body::Body,
    extract::{DefaultBodyLimit, Multipart, Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use serde_json::json;
use std::path::{PathBuf, Path as StdPath};
use tokio::fs::{File, create_dir_all};
use tokio::io::AsyncWriteExt;
use tokio_util::io::ReaderStream;
use uuid::Uuid;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route(
            "/compress/mp4",
            post(upload_mp4).layer(DefaultBodyLimit::disable()),
        )
        .route(
            "/compress/png",
            post(upload_png).layer(DefaultBodyLimit::disable()),
        )
        .route("/status/{id}", get(job_status))
        .route("/download/{id}", get(download_file))
        .with_state(state)
}

async fn upload_mp4(State(state): State<AppState>, mut multipart: Multipart) -> impl IntoResponse {
    let id = Uuid::new_v4().to_string();
    let _ = create_dir_all("uploads").await;

    let mut compression_level: u8 = 2; // padrão: Média
    let mut found_file = false;
    let mut saved_path = PathBuf::new();
    let mut saved_filename = String::new();
    let mut saved_original = String::new();

    while let Some(mut field) = match multipart.next_field().await {
        Ok(Some(f)) => Some(f),
        _ => None,
    } {
        let name = field.name().unwrap_or("").to_string();

        if name == "compression_level" {
            if let Ok(Some(chunk)) = field.chunk().await {
                if let Ok(val_str) = std::str::from_utf8(&chunk) {
                    compression_level = val_str.trim().parse().unwrap_or(2).clamp(1, 3);
                }
            }
            continue;
        }

        if name == "file" {
            found_file = true;
            let file_name = field.file_name().unwrap_or("video.mp4").to_string();
            saved_original = file_name.clone();
            saved_filename = file_name.clone();

            let path = format!("uploads/{}_{}", id, file_name);
            saved_path = PathBuf::from(&path);

            let mut disk_file = match File::create(&saved_path).await {
                Ok(f) => f,
                Err(e) => return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": format!("Erro ao criar arquivo: {}", e)})),
                ).into_response(),
            };

            while let Some(chunk) = match field.chunk().await {
                Ok(c) => c,
                Err(e) => return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": format!("Erro ao ler upload: {}", e)})),
                ).into_response(),
            } {
                if let Err(e) = disk_file.write_all(&chunk).await {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"error": format!("Erro ao escrever no disco: {}", e)})),
                    ).into_response();
                }
            }
        }
    }

    if !found_file {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "Nenhum arquivo encontrado no envio."}))).into_response();
    }

    // Registrar Job
    let mut s = state.write().await;
    s.jobs.insert(
        id.clone(),
        crate::state::JobStatus {
            id: id.clone(),
            status: "processing".to_string(),
            error: None,
            filename: saved_filename,
            compressed_filename: None,
        },
    );
    drop(s);

    // Iniciar compressão em background
    let state_clone = state.clone();
    let id_clone = id.clone();
    let temp_path = saved_path;
    let original_filename = saved_original;
    tokio::spawn(async move {
        // Obter nome de nivel
        let level_name = match compression_level {
            1 => "Leve",
            3 => "Alta",
            _ => "Média",
        };

        // Formatar no padrão antigo ou pegar extensao original
        let ext = StdPath::new(&original_filename)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("mp4");
        
        let file_stem = StdPath::new(&original_filename)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("video");

        let better_filename = format!("{}_{}.{}", file_stem, level_name, ext);
        let out_path = format!("uploads/{}_{}", id_clone, better_filename);

        let result = compress_media(
            MediaType::Mp4,
            &temp_path.to_string_lossy(),
            &out_path,
            compression_level,
        ).await;


        let mut s = state_clone.write().await;
        if let Some(job) = s.jobs.get_mut(&id_clone) {
            match result {
                Ok(_) => {
                    job.status = "completed".to_string();
                    job.compressed_filename = Some(out_path);
                }
                Err(e) => {
                    job.status = "error".to_string();
                    job.error = Some(e);
                }
            }
        }
    });

    (StatusCode::ACCEPTED, Json(json!({"id": id}))).into_response()
}

async fn upload_png(State(state): State<AppState>, mut multipart: Multipart) -> impl IntoResponse {
    let id = Uuid::new_v4().to_string();
    let _ = create_dir_all("uploads").await;

    let mut compression_level: u8 = 2;
    let mut found_file = false;
    let mut saved_path = PathBuf::new();
    let mut saved_filename = String::new();
    let mut saved_original = String::new();

    while let Some(mut field) = match multipart.next_field().await {
        Ok(Some(f)) => Some(f),
        _ => None,
    } {
        let name = field.name().unwrap_or("").to_string();

        if name == "compression_level" {
            if let Ok(Some(chunk)) = field.chunk().await {
                if let Ok(val_str) = std::str::from_utf8(&chunk) {
                    compression_level = val_str.trim().parse().unwrap_or(2).clamp(1, 3);
                }
            }
            continue;
        }

        if name == "file" {
            found_file = true;
            let file_name = field.file_name().unwrap_or("image.png").to_string();
            saved_original = file_name.clone();
            saved_filename = file_name.clone();

            let path = format!("uploads/{}_{}", id, file_name);
            saved_path = PathBuf::from(&path);

            let mut disk_file = match File::create(&saved_path).await {
                Ok(f) => f,
                Err(e) => return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": format!("Erro ao criar arquivo: {}", e)})),
                ).into_response(),
            };

            while let Some(chunk) = match field.chunk().await {
                Ok(c) => c,
                Err(e) => return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": format!("Erro ao ler upload: {}", e)})),
                ).into_response(),
            } {
                if let Err(e) = disk_file.write_all(&chunk).await {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"error": format!("Erro ao escrever no disco: {}", e)})),
                    ).into_response();
                }
            }
        }
    }

    if !found_file {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "Nenhum arquivo PNG encontrado no envio."}))).into_response();
    }

    let mut s = state.write().await;
    s.jobs.insert(
        id.clone(),
        crate::state::JobStatus {
            id: id.clone(),
            status: "processing".to_string(),
            error: None,
            filename: saved_filename,
            compressed_filename: None,
        },
    );
    drop(s);

    let state_clone = state.clone();
    let id_clone = id.clone();
    let temp_path = saved_path;
    let original_filename = saved_original;
    tokio::spawn(async move {
        // Obter nome de nivel
        let level_name = match compression_level {
            1 => "Leve",
            3 => "Alta",
            _ => "Média",
        };

        // Formatar no padrão antigo ou pegar extensao original
        let ext = StdPath::new(&original_filename)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("png");
        
        let file_stem = StdPath::new(&original_filename)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("image");

        let better_filename = format!("{}_{}.{}", file_stem, level_name, ext);
        let out_path = format!("uploads/{}_{}", id_clone, better_filename);

        let result = compress_media(
            MediaType::Png,
            &temp_path.to_string_lossy(),
            &out_path,
            compression_level,
        ).await;

        let mut s = state_clone.write().await;
        if let Some(job) = s.jobs.get_mut(&id_clone) {
            match result {
                Ok(_) => {
                    job.status = "completed".to_string();
                    job.compressed_filename = Some(out_path);
                }
                Err(e) => {
                    job.status = "error".to_string();
                    job.error = Some(e);
                }
            }
        }
    });

    (StatusCode::ACCEPTED, Json(json!({"id": id}))).into_response()
}

async fn job_status(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let s = state.read().await;
    if let Some(job) = s.jobs.get(&id) {
        return Json(job.clone()).into_response();
    }
    (StatusCode::NOT_FOUND, Json(json!({"error": "Job não encontrado"}))).into_response()
}

async fn download_file(State(state): State<AppState>, Path(id): Path<String>) -> Response {
    let s = state.read().await;
    let job = s.jobs.get(&id).cloned();
    drop(s);

    if let Some(job) = job {
        if let Some(compressed) = job.compressed_filename {
            if let Ok(file) = File::open(&compressed).await {
                // Pegar apenas o nome do arquivo ignorando a pasta uploads e id
                let file_name = StdPath::new(&compressed)
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("download");
                
                // Limpar regex / id do nome pra ficar bonitinho
                // O nome tá formatado como uploads/{id}_{nome_nivel}.{ext}, vamos arrancar id_
                let clean_name = if file_name.starts_with(&format!("{}_", id)) {
                    &file_name[id.len() + 1..]
                } else {
                    file_name
                };

                let content_type = if clean_name.ends_with(".png") {
                    "image/png"
                } else {
                    "video/mp4"
                };

                let stream = ReaderStream::new(file);
                let body = Body::from_stream(stream);
                return Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, content_type)
                    .header(header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", clean_name))
                    .body(body)
                    .unwrap();
            }
        }
    }

    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("Arquivo não encontrado"))
        .unwrap()
}
