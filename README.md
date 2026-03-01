# Conversor-Tools | Compressor de Mídia 🚀

O **Conversor-Tools** é um compressor web de alta performance construído com **Rust (Axum)** no backend e **JavaScript vanilla + CSS Moderno** no frontend. Ele foca em extrair a máxima eficiência de compressão de arquivos (Video MP4 e Imagem PNG) usando processamento em segundo plano e, quando disponível, aceleração de hardware (GPU AMD) no Linux.

Sua interface entrega a melhor experiência de usuário (UX) através de uma abordagem "Glassmorphism" polida, com prevenções ativas de cliques duplos, upload resiliente e processamento assíncrono (polling).

## Funcionalidades Principais 🌟

- **Compressão MP4 Otimizada:** Escolha o nível de compressão desejado. Suporta **aceleração GPU via AMD VAAPI** nativamente em servidores Linux (processamento brutalmente mais rápido para vídeos H.264).
- **Compressão PNG Inteligente:** Não confia no zlib — usa nativamente **`pngquant`** para realizar quantização lossy com até 80% de redução mantendo ótima fidelidade visual.
- **Nível "Extrema" (Panic Mode):** Um quarto nível de compressão destrutor de pixels. Feito para forçar o menor tamanho de bytes possível, esmagando a qualidade globalmente.
- **API Resiliente:** Upload multithread, sistema de polling de jobs baseado em ID (`UUID`) não obstrutivo no frontend com Auto-Backoff para economizar requisições do servidor.
- **Prevenção de Erros UX:** Bloqueia interações acidentais, reconecta sozinho se a rede cair durante a compressão e traduz falhas técnicas do `stderr` em português natural.

## Níveis de Compressão

| Nível | MP4 (GPU VAAPI) | MP4 (CPU x264) | PNG (`pngquant`) |
| --- | --- | --- | --- |
| 🟢 **Leve** | Qualidade: Alta (QP 22) | CRF 22 (Preset Fast) | Visual: 65%~80% |
| 🟡 **Média** | Qualidade: Normal (QP 28) | CRF 28 (Preset Fast) | Visual: 40%~60% |
| 🔴 **Alta** | Qualidade: Baixa (QP 35) | CRF 35 (Preset Fast) | Visual: 15%~35% |
| ⚫ **Extrema** | Dane-se a qualidade (Scale 0.5x, Audio 64k) | CRF 51 (Preset VerySlow, Scale 0.5x) | Qualidade 0~10 + Posterize brutal (Color Banding) |

---

## Como Instalar e Rodar (Servidor Ubuntu/Debian) 🐧

### 1. Dependências do Sistema Operacional
Para que o conversor consiga realizar sua mágica, o backend depende fortemente dos binários de compressão no sistema.

Rode o comando abaixo no terminal do seu servidor:
```bash
sudo apt update
sudo apt install -y ffmpeg pngquant
```

**Para Aceleração GPU AMD (VAAPI):**
Geralmente os pacotes modernos do Linux já têm suporte VAAPI pela mesa drivers. Verifique se seu sistema reconhece a renderização:
```bash
sudo apt install -y vainfo
vainfo
```
*(Certifique-se de que o device `/dev/dri/renderD128` está disponível no ambiente).*

### 2. Dependências do Rust
Se você não tem o ambiente Rust `cargo` instalado no servidor, instale via `rustup`:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 3. Rodando a Aplicação
Clone o projeto na sua máquina servidora, mude para a pasta e execute:

```bash
cargo run --release
```

O projeto fará o build otimizado e subirá um servidor web.
Ele ficará ouvindo publicamente em **`http://0.0.0.0:3000`** (ou a porta interna/IP exposto na sua rede web local ou reversa de VPN). Acesse via navegador!

## O que posso ajustar? ⚙️
- Servidor e Portas: Edite `src/main.rs`.
- Lógica Visual (Tailoring visual): Toda tela e UX isolada está em `static/`.
- Limites Customizados: `src/compressor/mp4.rs` (para tunar qualidades CRF de vídeo).

## Licença 📝
Livre. Modifique e hospede à vontade para sua casa ou empresa.
