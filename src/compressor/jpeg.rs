/// Realiza a compressão de imagem JPEG usando o `jpegoptim`.
/// 
/// Diferente de re-encoding cego, `jpegoptim` comprime aplicando quality percentual
/// de forma muito robusta.
///
/// # Níveis de compressão (faixa de qualidade do jpegoptim -m):
/// - 1 (Leve): 85% — excelente qualidade, boa redução
/// - 2 (Média): 65% — equilíbrio entre qualidade e tamanho
/// - 3 (Alta):  40% — máxima redução aceitável visualmente
/// - 4 (Extrema): 15% — máxima compressão para tamanho brutalmente pequeno
///
/// **Requisito**: `jpegoptim` instalado no servidor (`sudo apt install jpegoptim`).
pub async fn compress(input: &str, output: &str, level: u8) -> Result<(), String> {
    let quality = match level {
        1 => "85",
        3 => "40",
        4 => "15",
        _ => "65",
    };

    println!("[JPEG] Iniciando compressão com jpegoptim (qualidade máxima: {})...", quality);

    // O jpegoptim comprime in-place. Vamos copiar o original para o path de destino primeiro.
    tokio::fs::copy(input, output)
        .await
        .map_err(|e| format!("Falha ao copiar arquivo JPEG base: {}", e))?;

    let result = tokio::process::Command::new("jpegoptim")
        .args([
            "-m", quality,
            "--strip-all", // remove EXIF e metadados pesados
            output,
        ])
        .output()
        .await;

    match result {
        Ok(cmd) if cmd.status.success() => {
            println!("[JPEG] ✅ Compressão finalizada com sucesso (qualidade: {})!", quality);
            Ok(())
        }
        Ok(cmd) => {
            let code = cmd.status.code().unwrap_or(-1);
            let stderr = String::from_utf8_lossy(&cmd.stderr).to_string();
            // Erros normais de processo
            Err(format!("Falha na compressão JPEG (código {}): {}", code, stderr.lines().last().unwrap_or("desconhecido")))
        }
        Err(e) => {
            println!("[JPEG] ❌ jpegoptim não encontrado ou falha ao executar: {}", e);
            Err(format!(
                "jpegoptim não está instalado no servidor. Instale com: sudo apt install jpegoptim. Erro: {}",
                e
            ))
        }
    }
}
