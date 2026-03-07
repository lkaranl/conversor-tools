use tokio::process::Command;
use std::path::Path;

pub async fn compress(
    input: &str,
    output: &str,
    level: u8,
    progress_tx: Option<tokio::sync::mpsc::UnboundedSender<f32>>,
) -> Result<(), String> {
    // Determinar bitrate com base no nível escolhido
    // Leve (1): 192k
    // Média (2): 128k
    // Alta(3): 64k
    // Extrema(4): 32k
    let bitrate = match level {
        1 => "192k",
        3 => "64k",
        4 => "32k",
        _ => "128k", // default/2
    };

    let ext = Path::new(input)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("mp3")
        .to_lowercase();

    let mut args = vec![
        "-y",
        "-i", input,
        "-b:a", bitrate,
        "-vn",          // Remove capa/vídeo caso tenha pego junto
    ];

    if ext == "m4a" {
        args.push("-c:a");
        args.push("aac");
    } else {
        // Assume MP3 como codificador fallback normal
        args.push("-c:a");
        args.push("libmp3lame");
    }

    args.push(output);

    println!("[FFmpeg Audio] Rodando: ffmpeg {}", args.join(" "));

    let mut cmd = Command::new("ffmpeg");
    cmd.args(&args);

    let result = crate::utils::ffmpeg_progress::run_and_stream(&mut cmd, &progress_tx).await;

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Erro no FFmpeg: {}", e)),
    }
}
