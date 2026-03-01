/// Realiza a compressão do arquivo MP4 usando o FFmpeg.
///
/// Tenta primeiro usar aceleração por hardware AMD (h264_amf).
/// Se o encoder AMF não estiver disponível, faz fallback automático para CPU (libx264).
///
/// # Níveis de compressão:
/// - **AMF (GPU)**: usa quality level (menor = melhor qualidade)
///   - 1 (Leve): quality 22
///   - 2 (Média): quality 28
///   - 3 (Alta): quality 35
/// - **libx264 (CPU fallback)**: usa CRF (Constant Rate Factor) com os mesmos valores
pub async fn compress(input: &str, output: &str, level: u8) -> Result<(), String> {
    let quality = match level {
        1 => "22",
        3 => "35",
        _ => "28",
    };

    // --- Tentativa 1: GPU AMD (h264_amf) ---
    println!("[GPU] Tentando compressão com aceleração de hardware AMD (h264_amf)...");

    let gpu_result = tokio::process::Command::new("ffmpeg")
        .args([
            "-y",
            "-i", input,
            "-c:v", "h264_amf",
            "-quality", quality,
            "-threads", "0",
            "-movflags", "+faststart",
            output,
        ])
        .output()
        .await;

    match gpu_result {
        Ok(cmd) if cmd.status.success() => {
            println!("[GPU] ✅ Compressão finalizada com sucesso usando GPU AMD (h264_amf)!");
            return Ok(());
        }
        Ok(cmd) => {
            let stderr = String::from_utf8_lossy(&cmd.stderr);
            println!("[GPU] ⚠️  Falha ao usar h264_amf. Motivo: {}", stderr.lines().last().unwrap_or("desconhecido"));
        }
        Err(e) => {
            println!("[GPU] ⚠️  Não foi possível executar o FFmpeg com h264_amf: {}", e);
        }
    }

    // --- Fallback: CPU (libx264) ---
    println!("[CPU] Usando fallback com libx264 (CPU)...");

    let cpu_result = tokio::process::Command::new("ffmpeg")
        .args([
            "-y",
            "-i", input,
            "-c:v", "libx264",
            "-crf", quality,
            "-threads", "0",
            "-preset", "fast",
            "-movflags", "+faststart",
            output,
        ])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if cpu_result.status.success() {
        println!("[CPU] ✅ Compressão finalizada com sucesso usando CPU (libx264).");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&cpu_result.stderr).to_string();
        println!("[CPU] ❌ Falha na compressão via CPU: {}", stderr.lines().last().unwrap_or("desconhecido"));
        Err(stderr)
    }
}
