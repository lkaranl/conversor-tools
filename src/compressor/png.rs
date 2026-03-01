/// Realiza a compressão de imagem PNG usando o FFmpeg (oxipng via oxipng ou ffmpeg optipng).
///
/// Como não há GPU encoding para PNG (é um formato lossless), a compressão é feita via CPU
/// usando o FFmpeg recomprimindo a imagem para aplicar a máxima compressão zlib.
///
/// # Níveis de compressão (compressão zlib — 0 a 9):
/// - 1 (Leve): nível 3 — mais rápido, arquivo um pouco menor
/// - 2 (Média): nível 6 — equilíbrio entre velocidade e tamanho (padrão)
/// - 3 (Alta):  nível 9 — máxima compressão, mais lento
pub async fn compress(input: &str, output: &str, level: u8) -> Result<(), String> {
    let compression_level = match level {
        1 => "3",
        3 => "9",
        _ => "6",
    };

    println!("[PNG] Iniciando compressão de PNG (nível zlib {})...", compression_level);

    let result = tokio::process::Command::new("ffmpeg")
        .args([
            "-y",
            "-i", input,
            "-compression_level", compression_level,
            output,
        ])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if result.status.success() {
        println!("[PNG] ✅ Compressão finalizada com sucesso (nível zlib {}).", compression_level);
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&result.stderr).to_string();
        println!("[PNG] ❌ Falha na compressão: {}", stderr.lines().last().unwrap_or("desconhecido"));
        Err(stderr)
    }
}
