pub mod mp4;
pub mod png;

pub enum MediaType {
    Mp4,
    Png,
    // Futuramente: Mp3, Jpeg...
}

pub async fn compress_media(media_type: MediaType, input: &str, output: &str, level: u8) -> Result<(), String> {
    match media_type {
        MediaType::Mp4 => mp4::compress(input, output, level).await,
        MediaType::Png => png::compress(input, output, level).await,
    }
}
