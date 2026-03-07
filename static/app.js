document.addEventListener('DOMContentLoaded', () => {
    // --- State ---
    let selectedType = 'mp4';
    let selectedAccept = 'video/mp4,.mp4';
    let compressionLevel = 2;
    let pendingFile = null;

    // --- Elements ---
    const fileInput = document.getElementById('file-input');
    const dropzone = document.getElementById('dropzone');
    const dropzoneDefault = document.getElementById('dropzone-default');
    const dropzonePreview = document.getElementById('dropzone-preview');
    const dropzoneHint = document.getElementById('dropzone-hint');
    const selectedFileName = document.getElementById('selected-file-name');
    const selectedFileSize = document.getElementById('selected-file-size');
    const changeFileBtn = document.getElementById('change-file-btn');
    const compressBtn = document.getElementById('compress-btn');
    const errorMsg = document.getElementById('error-message');
    const uploadSection = document.getElementById('upload-section');
    const processingSection = document.getElementById('processing-section');
    const resultSection = document.getElementById('result-section');
    const downloadBtn = document.getElementById('download-btn');
    const resetBtn = document.getElementById('reset-btn');
    const retryLevelBtn = document.getElementById('retry-level-btn');
    const finalFilename = document.getElementById('final-filename');
    const statusTitle = document.getElementById('status-title');
    const statusDesc = document.getElementById('status-desc');
    const spinner = document.querySelector('.spinner');
    const mediaProgress = document.getElementById('media-progress');
    const progressPercent = document.getElementById('progress-percent');

    const pillBtns = document.querySelectorAll('.pill-btn');
    const mediaCards = document.querySelectorAll('.media-card:not(.coming-soon)');

    const levelNames = { 1: 'Leve', 2: 'Média', 3: 'Alta', 4: 'Extrema' };
    // Textos de dica da dropzone por tipo
    const hints = {
        mp4: 'Aceita arquivos .mp4',
        png: 'Aceita arquivos .png',
        jpeg: 'Aceita arquivos .jpeg, .jpg',
        audio: 'Aceita arquivos .mp3, .m4a',
        pdf: 'Aceita arquivos .pdf',
    };

    // --- Media Type Selection ---
    mediaCards.forEach(card => {
        card.addEventListener('click', () => {
            mediaCards.forEach(c => c.classList.remove('active'));
            card.classList.add('active');
            selectedType = card.dataset.type;
            selectedAccept = card.dataset.accept;
            fileInput.accept = selectedAccept;
            dropzoneHint.textContent = hints[selectedType] || '';
            errorMsg.textContent = '';
            clearFile();
        });
    });

    // --- Compression Pills ---
    pillBtns.forEach(btn => {
        btn.addEventListener('click', () => {
            if (btn.disabled) return;
            pillBtns.forEach(b => b.classList.remove('active'));
            btn.classList.add('active');
            compressionLevel = parseInt(btn.dataset.level);
        });
    });

    // --- Drag & Drop ---
    ['dragenter', 'dragover', 'dragleave', 'drop'].forEach(ev => {
        dropzone.addEventListener(ev, e => { e.preventDefault(); e.stopPropagation(); });
    });
    ['dragenter', 'dragover'].forEach(ev => dropzone.addEventListener(ev, () => dropzone.classList.add('drag-active')));
    ['dragleave', 'drop'].forEach(ev => dropzone.addEventListener(ev, () => dropzone.classList.remove('drag-active')));

    dropzone.addEventListener('drop', e => {
        if (e.dataTransfer.files.length) handleFile(e.dataTransfer.files[0]);
    });

    // Só abre o seletor ao clicar no estado padrão (não no preview)
    dropzoneDefault.addEventListener('click', () => fileInput.click());

    fileInput.addEventListener('change', function () {
        if (this.files.length) handleFile(this.files[0]);
    });

    changeFileBtn.addEventListener('click', e => {
        e.stopPropagation();
        clearFile();
        fileInput.click();
    });

    // --- File Handling ---
    function handleFile(file) {
        errorMsg.textContent = '';
        const ext = file.name.split('.').pop().toLowerCase();
        if (selectedType === 'mp4' && ext !== 'mp4') {
            errorMsg.textContent = 'Por favor, selecione um arquivo MP4 válido.';
            return;
        }
        if (selectedType === 'png' && ext !== 'png') {
            errorMsg.textContent = 'Por favor, selecione um arquivo PNG válido.';
            return;
        }
        if (selectedType === 'jpeg' && ext !== 'jpeg' && ext !== 'jpg') {
            errorMsg.textContent = 'Por favor, selecione um arquivo JPEG (.jpg ou .jpeg) válido.';
            return;
        }
        if (selectedType === 'audio' && ext !== 'mp3' && ext !== 'm4a') {
            errorMsg.textContent = 'Por favor, selecione um arquivo de Áudio (.mp3 ou .m4a) válido.';
            return;
        }
        if (selectedType === 'pdf' && ext !== 'pdf') {
            errorMsg.textContent = 'Por favor, selecione um arquivo PDF (.pdf) válido.';
            return;
        }

        pendingFile = file;
        showFilePreview(file);
    }

    function showFilePreview(file) {
        selectedFileName.textContent = file.name;
        selectedFileSize.textContent = formatSize(file.size);
        dropzoneDefault.classList.add('hidden');
        dropzonePreview.classList.remove('hidden');
        compressBtn.classList.remove('hidden');
    }

    function clearFile() {
        pendingFile = null;
        fileInput.value = '';
        dropzoneDefault.classList.remove('hidden');
        dropzonePreview.classList.add('hidden');
        compressBtn.classList.add('hidden');
        errorMsg.textContent = '';
    }

    function formatSize(bytes) {
        if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
        return (bytes / (1024 * 1024)).toFixed(2) + ' MB';
    }

    // --- Botão Comprimir ---
    compressBtn.addEventListener('click', () => {
        if (!pendingFile) return;
        uploadFile(pendingFile);
    });

    // --- Upload com proteção de double-submit ---
    async function uploadFile(file) {
        // Previne duplo clique e bloqueia UI
        compressBtn.disabled = true;
        compressBtn.textContent = 'Enviando...';
        toggleUI(true);

        showSection(processingSection);
        statusTitle.textContent = 'Enviando arquivo...';
        statusDesc.textContent = file.name;

        // Reset visual do loading
        spinner.classList.remove('hidden');
        mediaProgress.classList.add('hidden');
        progressPercent.classList.add('hidden');
        mediaProgress.value = 0;
        progressPercent.textContent = '0%';

        const formData = new FormData();
        formData.append('file', file);
        formData.append('compression_level', compressionLevel.toString());

        try {
            const endpoint = `/api/compress/${selectedType}`;
            let response;
            try {
                response = await fetch(endpoint, { method: 'POST', body: formData });
            } catch (_) {
                throw new Error('Não foi possível conectar ao servidor. Verifique sua conexão.');
            }

            if (!response.ok) {
                const err = await response.json().catch(() => ({}));
                throw new Error(err.error || `Erro ao enviar arquivo (código ${response.status}).`);
            }

            const data = await response.json();
            statusTitle.textContent = 'Comprimindo arquivo...';
            statusDesc.textContent = `Nível: ${levelNames[compressionLevel]} — isso pode levar alguns instantes`;
            pollStatus(data.id, 0);
        } catch (e) {
            handleError(e.message);
            // O desbloqueio da UI (toggleUI(false) e reset do botão)
            // foi movido para showSuccess() e handleError() para garantir
            // que os controles continuem bloqueados durante o polling.
        }
    }

    // --- Polling com timeout e backoff progressivo ---
    // Tenta por até ~3 minutos com backoff de 1.5s → 3s → 5s
    const MAX_POLLS = 80;

    async function pollStatus(jobId, attempts) {
        if (attempts >= MAX_POLLS) {
            handleError(
                'O processamento está demorando mais do que o esperado. ' +
                'O arquivo pode ser grande demais ou o servidor está sobrecarregado. ' +
                'Tente novamente com um arquivo menor.'
            );
            return;
        }

        // Backoff: começa rápido e vai desacelerando
        const delay = attempts < 10 ? 1500 : attempts < 30 ? 3000 : 5000;

        try {
            let res;
            try {
                res = await fetch(`/api/status/${jobId}`);
            } catch (_) {
                // Erro de rede transitório — tenta de novo com delay maior
                statusDesc.textContent = 'Reconectando ao servidor...';
                setTimeout(() => pollStatus(jobId, attempts + 1), 5000);
                return;
            }

            if (res.status === 404) {
                handleError(
                    'O job de compressão não foi encontrado. ' +
                    'Provavelmente o servidor foi reiniciado durante o processo. ' +
                    'Por favor, envie o arquivo novamente.'
                );
                return;
            }

            if (!res.ok) {
                throw new Error(`Erro ao verificar status (código ${res.status}).`);
            }

            const job = await res.json();

            // Lógica de Renderização do Progresso
            if (job.progress !== null && job.progress !== undefined) {
                spinner.classList.add('hidden');
                mediaProgress.classList.remove('hidden');
                progressPercent.classList.remove('hidden');

                mediaProgress.value = job.progress;
                progressPercent.textContent = `${Math.round(job.progress)}%`;
            }

            if (job.status === 'completed') {
                await showSuccess(job.id, job.filename);
            } else if (job.status === 'error') {
                // Humaniza erros técnicos do FFmpeg
                const rawError = job.error || '';
                const friendlyError = humanizeError(rawError);
                handleError(friendlyError);
            } else {
                setTimeout(() => pollStatus(jobId, attempts + 1), delay);
            }
        } catch (e) {
            handleError(e.message);
        }
    }

    // --- Transforma erros técnicos em mensagens amigáveis ---
    function humanizeError(raw) {
        if (!raw) return 'Erro desconhecido ao processar o arquivo.';
        if (raw.includes('No such file')) return 'O arquivo enviado não foi encontrado no servidor.';
        if (raw.includes('Invalid data') || raw.includes('moov atom')) return 'O arquivo enviado está corrompido ou em formato inválido.';
        if (raw.includes('Encoder not found')) return 'O encoder de vídeo não está disponível no servidor.';
        if (raw.includes('Permission denied')) return 'O servidor não tem permissão para processar este arquivo.';
        if (raw.includes('No space left')) return 'O servidor ficou sem espaço em disco durante a compressão.';
        // Fallback: mostra apenas última linha (menos técnica)
        const lastLine = raw.trim().split('\n').pop();
        return `Falha na compressão: ${lastLine}`;
    }

    // --- Sucesso: verifica disponibilidade do download antes de exibir ---
    async function showSuccess(jobId, filename) {
        const downloadUrl = `/api/download/${jobId}`;

        // Verifica se o arquivo realmente está acessível antes de mostrar
        try {
            const check = await fetch(downloadUrl, { method: 'HEAD' });
            if (!check.ok) {
                handleError('A compressão foi concluída, mas o arquivo resultante não está disponível para download. Tente novamente.');
                return;
            }
        } catch (_) {
            // Se não conseguir verificar, ainda assim mostra (pode ser CORS/HEAD bloqueado)
        }

        showSection(resultSection);

        // Formata o novo nome do arquivo: ex "video_Leve.mp4"
        const currentLevelName = levelNames[compressionLevel];
        const nameParts = pendingFile.name.split('.');
        const ext = nameParts.pop();
        const baseName = nameParts.join('.');
        const betterFilename = `${baseName}_${currentLevelName}.${ext}`;

        // Restaura botão de Comprimir mas mantém opções (Níveis) bloqueados na tela de fundo
        resetCompressBtnUI(true);

        finalFilename.textContent = betterFilename;
        downloadBtn.href = downloadUrl;
        downloadBtn.setAttribute('download', betterFilename);
    }

    function handleError(msg) {
        showSection(uploadSection);
        errorMsg.textContent = `⚠ ${msg}`;
        // Reativa os controles, pois como é um erro, a pessoa precisa poder tentar de novo imediatamente
        resetCompressBtnUI(false);
    }

    // Helper para restaurar o visual do botão Comprimir. 
    // keepLevelsLocked mantém a seção de configurações cinza/inativa enquanto o usuário está na tela de resultado
    function resetCompressBtnUI(keepLevelsLocked = false) {
        compressBtn.disabled = false;
        toggleUI(false, keepLevelsLocked);
        compressBtn.innerHTML = `
            <svg style="width:16px;height:16px;vertical-align:-3px;margin-right:6px" viewBox="0 0 24 24"
                fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"
                stroke-linejoin="round">
                <polyline points="8 17 12 21 16 17" />
                <line x1="12" y1="3" x2="12" y2="21" />
            </svg>
            Comprimir Agora`;
    }

    function showSection(section) {
        [uploadSection, processingSection, resultSection].forEach(el => el.classList.add('hidden'));
        section.classList.remove('hidden');
    }

    function toggleUI(disabled, keepLevelsLocked = false) {
        // Obter os containers para adicionar efeito de "desativado" em bloco
        const typeSection = document.querySelector('.media-type-section');
        const compSection = document.querySelector('.compression-section');

        const stateForLevels = disabled || keepLevelsLocked;

        [typeSection, compSection].forEach(sec => {
            if (!sec) return;
            sec.style.pointerEvents = stateForLevels ? 'none' : 'auto';
            sec.style.opacity = stateForLevels ? '0.4' : '1';
            // Para mostrar o cursor correto sobre a área desativada
            sec.style.cursor = stateForLevels ? 'not-allowed' : 'auto';
        });

        // Garantir que as pílulas e cards internos fiquem inertes independente do pai
        pillBtns.forEach(btn => btn.disabled = stateForLevels);

        changeFileBtn.disabled = disabled; // O botão de trocar arquivo é liberado pelo `toggleUI(false)` basico
        changeFileBtn.style.opacity = disabled ? '0.4' : '1';
        changeFileBtn.style.cursor = disabled ? 'not-allowed' : 'pointer';
    }

    resetBtn.addEventListener('click', () => {
        clearFile();
        toggleUI(false); // Libera tudo para o próximo arquivo novo
        showSection(uploadSection);
    });

    retryLevelBtn.addEventListener('click', () => {
        // Volta pra tela de upload, mas não limpa o pendingFile
        statusTitle.textContent = 'Processando arquivo...'; // reset texto padrão
        statusDesc.textContent = '';

        // Agora sim destravamos a UI para que ele possa clicar nos níveis:
        toggleUI(false);

        showSection(uploadSection);
    });
});

