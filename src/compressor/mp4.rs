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
    let (quality, preset) = match level {
        1 => ("22", "fast"),
        3 => ("35", "fast"),
        4 => ("51", "veryslow"), // Nível Extremo: CRF máximo do H.264 + máxima eficiência
        _ => ("28", "fast"),
    };

    let is_extreme = level == 4;

    // --- Tentativa 1: GPU AMD via VAAPI (h264_vaapi) ---
    println!("[GPU] Tentando compressão com aceleração de hardware AMD via VAAPI (h264_vaapi)...");

    // Para VAAPI no nível extremo, adicionamos scale de metade da resolução
    let vf_filter = if is_extreme {
        "format=nv12,hwupload,scale_vaapi=w=iw/2:h=ih/2"
    } else {
        "format=nv12,hwupload"
    };

    let mut gpu_args: Vec<&str> = vec![
        "-y",
        "-vaapi_device", "/dev/dri/renderD128",
        "-i", input,
        "-vf", vf_filter,
        "-c:v", "h264_vaapi",
        "-qp", quality,
        "-threads", "0",
        "-movflags", "+faststart",
    ];
    // Áudio agressivo no extremo
    if is_extreme {
        gpu_args.extend_from_slice(&["-c:a", "aac", "-b:a", "64k"]);
    }
    gpu_args.push(output);

    let gpu_result = tokio::process::Command::new("ffmpeg")
        .args(&gpu_args)
        .output()
        .await;

    match gpu_result {
        Ok(cmd) if cmd.status.success() => {
            println!("[GPU] ✅ Compressão finalizada com sucesso usando GPU AMD via VAAPI!");
            return Ok(());
        }
        Ok(cmd) => {
            let stderr = String::from_utf8_lossy(&cmd.stderr);
            println!("[GPU] ⚠️  Falha ao usar h264_vaapi. Motivo: {}", stderr.lines().last().unwrap_or("desconhecido"));
        }
        Err(e) => {
            println!("[GPU] ⚠️  Não foi possível executar o FFmpeg com h264_vaapi: {}", e);
        }
    }

    // --- Fallback: CPU (libx264) ---
    println!("[CPU] Usando fallback com libx264 (CPU)...");

    // Para CPU no nível extremo, adicionamos scale de metade da resolução
    let mut cpu_args: Vec<&str> = vec![
        "-y",
        "-i", input,
    ];
    if is_extreme {
        cpu_args.extend_from_slice(&["-vf", "scale=iw/2:ih/2"]);
    }
    cpu_args.extend_from_slice(&[
        "-c:v", "libx264",
        "-crf", quality,
        "-threads", "0",
        "-preset", preset,
        "-movflags", "+faststart",
    ]);
    if is_extreme {
        cpu_args.extend_from_slice(&["-c:a", "aac", "-b:a", "64k"]);
    }
    cpu_args.push(output);

    let cpu_result = tokio::process::Command::new("ffmpeg")
        .args(&cpu_args)
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

