pub mod mp4;
pub mod png;
pub mod jpeg;
pub mod audio;
pub mod pdf;

pub enum MediaType {
    Mp4,
    Png,
    Jpeg,
    Audio,
    Pdf,
}

pub async fn compress_media(
    media_type: MediaType,
    input: &str,
    output: &str,
    level: u8,
    progress_tx: Option<tokio::sync::mpsc::UnboundedSender<f32>>,
) -> Result<(), String> {
    match media_type {
        MediaType::Mp4 => mp4::compress(input, output, level, progress_tx).await,
        MediaType::Png => png::compress(input, output, level).await,
        MediaType::Jpeg => jpeg::compress(input, output, level).await,
        MediaType::Audio => audio::compress(input, output, level, progress_tx).await,
        MediaType::Pdf => pdf::compress(input, output, level).await,
    }
}
