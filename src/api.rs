use crate::state::AppState;
use crate::compressor::{compress_media, MediaType};
use axum::{
    body::Body,
    extract::{DefaultBodyLimit, Multipart, Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use serde_json::json;
use std::path::{PathBuf, Path as StdPath};
use tokio::fs::{File, create_dir_all};
use tokio::io::AsyncWriteExt;
use tokio_util::io::ReaderStream;
use uuid::Uuid;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route(
            "/compress",
            post(upload_batch).layer(DefaultBodyLimit::disable()),
        )
        .route(
            "/compress/mp4",
            post(upload_mp4).layer(DefaultBodyLimit::disable()),
        )
        .route(
            "/compress/png",
            post(upload_png).layer(DefaultBodyLimit::disable()),
        )
        .route(
            "/compress/jpeg",
            post(upload_jpeg).layer(DefaultBodyLimit::disable()),
        )
        .route(
            "/compress/audio",
            post(upload_audio).layer(DefaultBodyLimit::disable()),
        )
        .route(
            "/compress/pdf",
            post(upload_pdf).layer(DefaultBodyLimit::disable()),
        )
        .route("/download/zip", get(download_batch_zip))
        .route("/status/{id}", get(job_status))
        .route("/download/{id}", get(download_file))
        .with_state(state)
}

fn detect_media_type(filename: &str) -> Option<MediaType> {
    let ext = StdPath::new(filename)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "mp4" => Some(MediaType::Mp4),
        "png" => Some(MediaType::Png),
        "jpg" | "jpeg" => Some(MediaType::Jpeg),
        "mp3" | "m4a" => Some(MediaType::Audio),
        "pdf" => Some(MediaType::Pdf),
        _ => None,
    }
}

async fn start_compression_job(
    state: AppState,
    id: String,
    media_type: MediaType,
    saved_path: PathBuf,
    original_filename: String,
    compression_level: u8,
) {
    let state_clone = state.clone();
    let id_clone = id.clone();
    let temp_path = saved_path;
    let original_filename_clone = original_filename.clone();

    tokio::spawn(async move {
        let level_name = match compression_level {
            1 => "Leve",
            3 => "Alta",
            4 => "Extrema",
            _ => "Média",
        };

        let ext = StdPath::new(&original_filename_clone)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or(match media_type {
                MediaType::Mp4 => "mp4",
                MediaType::Png => "png",
                MediaType::Jpeg => "jpg",
                MediaType::Audio => "mp3",
                MediaType::Pdf => "pdf",
            });

        let file_stem = StdPath::new(&original_filename_clone)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("file");

        let better_filename = format!("{}_{}.{}", file_stem, level_name, ext);
        let out_path = format!("uploads/{}_{}", id_clone, better_filename);

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<f32>();

        let state_progress = state_clone.clone();
        let id_progress = id_clone.clone();
        tokio::spawn(async move {
            while let Some(progress) = rx.recv().await {
                if let Some(job) = state_progress.write().await.jobs.get_mut(&id_progress) {
                    if let Some(p) = job.progress {
                        if progress > p || (progress == 0.0) {
                            job.progress = Some(progress);
                        }
                    } else {
                        job.progress = Some(progress);
                    }
                }
            }
        });

        let has_progress = matches!(media_type, MediaType::Mp4 | MediaType::Audio);

        let result = compress_media(
            media_type,
            &temp_path.to_string_lossy(),
            &out_path,
            compression_level,
            if has_progress { Some(tx) } else { None },
        )
        .await;

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
}

async fn upload_batch(State(state): State<AppState>, mut multipart: Multipart) -> impl IntoResponse {
    let _ = create_dir_all("uploads").await;
    let mut compression_level: u8 = 2;
    let mut jobs_started = Vec::new();

    // Primeiro passamos para pegar o nível de compressão se ele vier primeiro, 
    // ou guardamos os arquivos para processar depois.
    // Mas Multipart em Axum é sequencial. Então vamos processar conforme vier.
    
    while let Some(mut field) = match multipart.next_field().await {
        Ok(Some(f)) => Some(f),
        _ => None,
    } {
        let name = field.name().unwrap_or("").to_string();

        if name == "compression_level" {
            if let Ok(Some(chunk)) = field.chunk().await {
                if let Ok(val_str) = std::str::from_utf8(&chunk) {
                    compression_level = val_str.trim().parse().unwrap_or(2).clamp(1, 4);
                }
            }
            continue;
        }

        if name == "file" {
            let id = Uuid::new_v4().to_string();
            let original_filename = field.file_name().unwrap_or("file").to_string();
            
            let media_type = match detect_media_type(&original_filename) {
                Some(t) => t,
                None => continue, // Pula arquivos não suportados silenciosamente ou tratar erro
            };

            let path = format!("uploads/{}_{}", id, original_filename);
            let saved_path = PathBuf::from(&path);

            let mut disk_file = match File::create(&saved_path).await {
                Ok(f) => f,
                Err(_) => continue,
            };

            let mut error_occurred = false;
            loop {
                match field.chunk().await {
                    Ok(Some(chunk)) => {
                        if disk_file.write_all(&chunk).await.is_err() {
                            error_occurred = true;
                            break;
                        }
                    }
                    Ok(None) => break,
                    Err(_) => {
                        error_occurred = true;
                        break;
                    }
                }
            }

            if error_occurred {
                continue;
            }

            // Registrar Job
            let mut s = state.write().await;
            s.jobs.insert(
                id.clone(),
                crate::state::JobStatus {
                    id: id.clone(),
                    status: "processing".to_string(),
                    progress: if matches!(media_type, MediaType::Mp4 | MediaType::Audio) { Some(0.0) } else { None },
                    error: None,
                    filename: original_filename.clone(),
                    compressed_filename: None,
                },
            );
            drop(s);

            start_compression_job(
                state.clone(),
                id.clone(),
                media_type,
                saved_path,
                original_filename,
                compression_level,
            ).await;

            jobs_started.push(id);
        }
    }

    if jobs_started.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "Nenhum arquivo válido encontrado no envio."}))).into_response();
    }

    (StatusCode::ACCEPTED, Json(json!({"ids": jobs_started}))).into_response()
}

async fn upload_mp4(state: State<AppState>, multipart: Multipart) -> impl IntoResponse {
    upload_batch(state, multipart).await
}

async fn upload_png(state: State<AppState>, multipart: Multipart) -> impl IntoResponse {
    upload_batch(state, multipart).await
}

async fn upload_jpeg(state: State<AppState>, multipart: Multipart) -> impl IntoResponse {
    upload_batch(state, multipart).await
}

async fn upload_audio(state: State<AppState>, multipart: Multipart) -> impl IntoResponse {
    upload_batch(state, multipart).await
}

async fn upload_pdf(state: State<AppState>, multipart: Multipart) -> impl IntoResponse {
    upload_batch(state, multipart).await
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
                } else if clean_name.ends_with(".jpg") || clean_name.ends_with(".jpeg") {
                    "image/jpeg"
                } else if clean_name.ends_with(".mp3") {
                    "audio/mpeg"
                } else if clean_name.ends_with(".m4a") {
                    "audio/mp4"
                } else if clean_name.ends_with(".pdf") {
                    "application/pdf"
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

#[derive(Deserialize)]
pub struct ZipParams {
    pub ids: String,
}

async fn download_batch_zip(
    State(state): State<AppState>,
    Query(params): Query<ZipParams>,
) -> impl IntoResponse {
    let ids: Vec<&str> = params.ids.split(',').collect();
    
    let mut buffer = Vec::new();
    let mut zip_writer = zip::ZipWriter::new(std::io::Cursor::new(&mut buffer));
    
    let mut files_added = 0;
    
    {
        let state_read = state.read().await;
        for id in ids {
            if let Some(job) = state_read.jobs.get(id) {
                if job.status == "completed" {
                    if let Some(ref compressed) = job.compressed_filename {
                        // compressed_filename already contains "uploads/" prefix
                        let path = compressed; 
                        if let Ok(mut file) = std::fs::File::open(path) {
                            let options = zip::write::SimpleFileOptions::default()
                                .compression_method(zip::CompressionMethod::Deflated);
                            
                            // Strip "uploads/" prefix for the ZIP entry name
                            let mut entry_name = compressed.as_str();
                            if entry_name.starts_with("uploads/") {
                                entry_name = &entry_name[8..];
                            }
                            // Also strip UUID prefix (36 chars + 1 underscore = 37)
                            if entry_name.len() > 37 {
                                entry_name = &entry_name[37..];
                            }

                            if zip_writer.start_file(entry_name, options).is_ok() {
                                if std::io::copy(&mut file, &mut zip_writer).is_ok() {
                                    files_added += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    if files_added == 0 {
        return (StatusCode::NOT_FOUND, "Nenhum arquivo finalizado encontrado para o ZIP").into_response();
    }
    
    if let Ok(_) = zip_writer.finish() {
        Response::builder()
            .header(header::CONTENT_TYPE, "application/zip")
            .header(header::CONTENT_DISPOSITION, "attachment; filename=\"arquivos_comprimidos.zip\"")
            .body(Body::from(buffer))
            .unwrap()
            .into_response()
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "Erro ao gerar arquivo ZIP").into_response()
    }
}
