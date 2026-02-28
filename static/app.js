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
    const finalFilename = document.getElementById('final-filename');
    const statusTitle = document.getElementById('status-title');
    const statusDesc = document.getElementById('status-desc');
    const pillBtns = document.querySelectorAll('.pill-btn');
    const mediaCards = document.querySelectorAll('.media-card:not(.coming-soon)');

    const levelNames = { 1: 'Leve', 2: 'Média', 3: 'Alta' };
    const hints = {
        mp4: 'Aceita arquivos .mp4',
        mp3: 'Aceita arquivos .mp3',
        png: 'Aceita arquivos .png',
        jpeg: 'Aceita arquivos .jpg, .jpeg',
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

    async function uploadFile(file) {
        showSection(processingSection);
        statusTitle.textContent = 'Enviando arquivo...';
        statusDesc.textContent = file.name;

        const formData = new FormData();
        formData.append('file', file);
        formData.append('compression_level', compressionLevel.toString());

        try {
            const endpoint = `/api/compress/${selectedType}`;
            const response = await fetch(endpoint, { method: 'POST', body: formData });
            if (!response.ok) {
                const err = await response.json().catch(() => ({ error: response.statusText }));
                throw new Error(err.error || 'Erro no servidor.');
            }
            const data = await response.json();
            statusTitle.textContent = 'Comprimindo com FFmpeg...';
            statusDesc.textContent = `Nível: ${levelNames[compressionLevel]}`;
            pollStatus(data.id);
        } catch (e) {
            handleError(e.message);
        }
    }

    async function pollStatus(jobId) {
        try {
            const res = await fetch(`/api/status/${jobId}`);
            if (!res.ok) throw new Error('Job não encontrado.');
            const job = await res.json();
            if (job.status === 'completed') {
                showSuccess(job.id, job.filename);
            } else if (job.status === 'error') {
                handleError(job.error || 'Erro ao processar o arquivo.');
            } else {
                setTimeout(() => pollStatus(jobId), 1500);
            }
        } catch (e) {
            handleError('A conexão com o servidor foi perdida.');
        }
    }

    function showSuccess(jobId, filename) {
        showSection(resultSection);
        finalFilename.textContent = filename;
        downloadBtn.href = `/api/download/${jobId}`;
        downloadBtn.setAttribute('download', `compressed_${filename}`);
    }

    function handleError(msg) {
        showSection(uploadSection);
        errorMsg.textContent = `⚠ ${msg}`;
        fileInput.value = '';
    }

    function showSection(section) {
        [uploadSection, processingSection, resultSection].forEach(el => el.classList.add('hidden'));
        section.classList.remove('hidden');
    }

    resetBtn.addEventListener('click', () => {
        clearFile();
        showSection(uploadSection);
    });
});
