use std::process::Command;

pub async fn compress(input: &str, output: &str, level: u8) -> Result<(), String> {
    // Definimos a intenção de qualidade padrão do Ghostscript dPDFSETTINGS
    let (pdf_settings, convert_gray) = match level {
        1 => ("/printer", false), // 300 dpi (alta qualidade para impressão)
        3 => ("/screen", false),  // 72 dpi (leitura simples tela)
        4 => ("/screen", true),   // 72 dpi + Remoção forçada de cores (Tons de cinza)
        _ => ("/ebook", false),   // Média (150 dpi leitura ideal)
    };

    let pdf_settings_arg = format!("-dPDFSETTINGS={}", pdf_settings);
    let output_file_arg = format!("-sOutputFile={}", output);

    let mut args = vec![
        "-sDEVICE=pdfwrite",
        "-dCompatibilityLevel=1.4",
        &pdf_settings_arg,
        "-dNOPAUSE",
        "-dQUIET",
        "-dBATCH",
    ];

    if convert_gray {
        args.push("-sColorConversionStrategy=Gray");
        args.push("-dProcessColorModel=/DeviceGray");
    }

    args.push(&output_file_arg);
    args.push(input);

    println!("[Ghostscript] Rodando: gs {}", args.join(" "));

    let child = Command::new("gs")
        .args(&args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn();

    let output_result = match child {
        Ok(process) => process.wait_with_output(),
        Err(e) => return Err(format!("Falha ao iniciar Ghostscript (gs): {}", e)),
    };

    match output_result {
        Ok(out) => {
            if out.status.success() {
                Ok(())
            } else {
                let err_str = String::from_utf8_lossy(&out.stderr);
                Err(format!("Erro no Ghostscript: {}", err_str))
            }
        }
        Err(e) => Err(format!("Erro ao aguardar processo gs: {}", e)),
    }
}
