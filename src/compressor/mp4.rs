/// Realiza a compressão do arquivo MP4 usando o FFmpeg.
/// 
/// # Níveis de compressão (CRF — Constant Rate Factor):
/// - 1 (Leve): CRF 22 — alta qualidade visual, menor redução de tamanho
/// - 2 (Média): CRF 28 — equilíbrio entre qualidade e tamanho (padrão)
/// - 3 (Alta):  CRF 35 — menor tamanho possível, qualidade reduzida
pub async fn compress(input: &str, output: &str, level: u8) -> Result<(), String> {
    let crf = match level {
        1 => "22",
        3 => "35",
        _ => "28", // nível 2 ou qualquer outro valor
    };

    let output_cmd = tokio::process::Command::new("ffmpeg")
        .args([
            "-y",
            "-i", input,
            "-vcodec", "libx264",
            "-crf", crf,
            "-threads", "0",
            "-preset", "fast",
            "-movflags", "+faststart", // otimiza para streaming progressivo
            output,
        ])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if output_cmd.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output_cmd.stderr).to_string())
    }
}
