use regex::Regex;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc::UnboundedSender;

fn parse_duration(s: &str) -> Option<f32> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() == 3 {
        let h: f32 = parts[0].parse().unwrap_or(0.0);
        let m: f32 = parts[1].parse().unwrap_or(0.0);
        let s: f32 = parts[2].parse().unwrap_or(0.0);
        Some(h * 3600.0 + m * 60.0 + s)
    } else {
        None
    }
}

pub async fn run_and_stream(
    cmd: &mut Command,
    progress_tx: &Option<UnboundedSender<f32>>,
) -> Result<(), String> {
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| format!("Falha ao iniciar processo: {}", e))?;

    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "Falha ao capturar stderr do FFmpeg".to_string())?;

    let mut reader = tokio::io::BufReader::new(stderr);

    let re_duration = Regex::new(r"Duration: (\d{2}:\d{2}:\d{2}\.\d{2})").unwrap();
    let re_time = Regex::new(r"time=(\d{2}:\d{2}:\d{2}\.\d{2})").unwrap();

    let mut total_duration_sec: Option<f32> = None;
    let mut last_error_line = String::new();
    let mut buffer = Vec::new();

    loop {
        buffer.clear();
        // FFmpeg writes progress using carriage return '\r' and newlines '\n'
        let bytes_read = match reader.read_until(b'\r', &mut buffer).await {
            Ok(0) => break, // EOF
            Ok(n) => n,
            Err(_) => break,
        };

        let line = String::from_utf8_lossy(&buffer).to_string();
        
        // As FFmpeg occasionally mixes \n and \r, let's split by \n just in case,
        // though our primary delimiter is \r now.
        for chunk in line.split('\n') {
            let chunk = chunk.trim();
            if chunk.is_empty() { continue; }
            last_error_line = chunk.to_string();

            if total_duration_sec.is_none() {
                if let Some(caps) = re_duration.captures(chunk) {
                    if let Some(dur_str) = caps.get(1) {
                        total_duration_sec = parse_duration(dur_str.as_str());
                    }
                }
            }

            if let Some(total) = total_duration_sec {
                if let Some(caps) = re_time.captures(chunk) {
                    if let Some(time_str) = caps.get(1) {
                        if let Some(current_time) = parse_duration(time_str.as_str()) {
                            let mut progress = (current_time / total) * 100.0;
                            if progress > 100.0 {
                                progress = 100.0;
                            }

                            if let Some(tx) = progress_tx {
                                let _ = tx.send(progress);
                            }
                        }
                    }
                }
            }
        }
    }

    let status = child
        .wait()
        .await
        .map_err(|e| format!("Falha ao esperar pelo FFmpeg: {}", e))?;

    if status.success() {
        if let Some(tx) = progress_tx {
            let _ = tx.send(100.0);
        }
        Ok(())
    } else {
        Err(last_error_line)
    }
}
