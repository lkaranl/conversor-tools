pub mod mp4;

pub enum MediaType {
    Mp4,
    // Futuramente: Mp3, Png, Jpeg...
}

pub async fn compress_media(media_type: MediaType, input: &str, output: &str, level: u8) -> Result<(), String> {
    match media_type {
        MediaType::Mp4 => mp4::compress(input, output, level).await,
    }
}
