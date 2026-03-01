/// Realiza a compressão de imagem PNG usando o `pngquant`.
///
/// Diferente da abordagem anterior (FFmpeg com zlib), o `pngquant` faz quantização
/// de cores (lossy) — reduzindo inteligentemente a paleta de cores da imagem.
/// Isso resulta em reduções reais de 50-80% no tamanho do arquivo,
/// mantendo qualidade visual excelente.
///
/// # Níveis de compressão (faixa de qualidade do pngquant):
/// - 1 (Leve): 65–80 — excelente qualidade, boa redução (~30-50%)
/// - 2 (Média): 40–60 — equilíbrio entre qualidade e tamanho (~50-65%)
/// - 3 (Alta):  15–35 — máxima redução, qualidade aceitável (~65-80%)
/// - 4 (Extrema): 5-15 — dane-se a qualidade, menor tamanho possível
///
/// **Requisito**: `pngquant` instalado no servidor (`sudo apt install pngquant`).
pub async fn compress(input: &str, output: &str, level: u8) -> Result<(), String> {
    let quality = match level {
        1 => "65-80",
        3 => "15-35",
        4 => "0-10",
        _ => "40-60",
    };

    let is_extreme = level == 4;

    println!("[PNG] Iniciando compressão com pngquant (qualidade: {}{})...", quality, if is_extreme { " + posterize" } else { "" });

    let mut args: Vec<&str> = vec![
        "--quality", quality,
        "--force",
        "--output", output,
        "--speed", "1",
        "--strip",
    ];
    // Posterize reduz bits por canal, achatando gradientes brutalmente
    if is_extreme {
        args.extend_from_slice(&["--posterize", "4"]);
    }
    args.push(input);

    let result = tokio::process::Command::new("pngquant")
        .args(&args)
        .output()
        .await;

    match result {
        Ok(cmd) if cmd.status.success() => {
            println!("[PNG] ✅ Compressão finalizada com sucesso (qualidade: {})!", quality);
            Ok(())
        }
        Ok(cmd) => {
            let code = cmd.status.code().unwrap_or(-1);
            let stderr = String::from_utf8_lossy(&cmd.stderr).to_string();

            // Código 99 do pngquant = qualidade mínima não atingida (imagem já muito otimizada)
            if code == 99 {
                println!("[PNG] ⚠️  pngquant: imagem já está otimizada, copiando original...");
                tokio::fs::copy(input, output)
                    .await
                    .map_err(|e| format!("Falha ao copiar arquivo original: {}", e))?;
                return Ok(());
            }

            println!("[PNG] ❌ Falha no pngquant (código {}): {}", code, stderr.lines().last().unwrap_or("desconhecido"));
            Err(format!("Falha na compressão PNG: {}", stderr))
        }
        Err(e) => {
            println!("[PNG] ❌ pngquant não encontrado ou falha ao executar: {}", e);
            Err(format!(
                "pngquant não está instalado no servidor. Instale com: sudo apt install pngquant. Erro: {}",
                e
            ))
        }
    }
}

