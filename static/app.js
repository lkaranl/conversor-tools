document.addEventListener('DOMContentLoaded', () => {
    // --- State ---
    let compressionLevel = 2;
    let pendingFiles = [];
    let activeJobs = {}; // jobId -> { filename, status, progress, element }

    // --- Elements ---
    const fileInput = document.getElementById('file-input');
    const dropzone = document.getElementById('dropzone');
    const dropzoneDefault = document.getElementById('dropzone-default');
    const dropzonePreview = document.getElementById('dropzone-preview');
    const dropzoneHint = document.getElementById('dropzone-hint');
    const selectedFilesList = document.getElementById('selected-files-list');
    const addMoreBtn = document.getElementById('add-more-btn');
    const clearAllBtn = document.getElementById('clear-all-btn');
    const compressBtn = document.getElementById('compress-btn');
    const errorMsg = document.getElementById('error-message');
    
    const uploadSection = document.getElementById('upload-section');
    const statusSection = document.getElementById('status-section');
    const jobsList = document.getElementById('jobs-list');
    const batchStatusTitle = document.getElementById('batch-status-title');
    const batchProgressBar = document.getElementById('batch-progress-bar');
    const batchProgressText = document.getElementById('batch-progress-text');
    const batchActions = document.getElementById('batch-actions');
    const resetBtn = document.getElementById('reset-btn');

    const pillBtns = document.querySelectorAll('.pill-btn');

    const levelNames = { 1: 'Leve', 2: 'Média', 3: 'Alta', 4: 'Extrema' };
    
    // Icons mapping
    const fileIcons = {
        mp4: '📹',
        png: '🖼️',
        jpeg: '🖼️',
        jpg: '🖼️',
        mp3: '🎵',
        m4a: '🎵',
        pdf: '📄',
        default: '📄'
    };

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
        if (e.dataTransfer.files.length) handleFiles(e.dataTransfer.files);
    });

    dropzoneDefault.addEventListener('click', () => fileInput.click());
    addMoreBtn.addEventListener('click', () => fileInput.click());

    fileInput.addEventListener('change', function () {
        if (this.files.length) handleFiles(this.files);
    });

    clearAllBtn.addEventListener('click', () => {
        pendingFiles = [];
        renderFileList();
    });

    // --- File Handling ---
    function handleFiles(files) {
        errorMsg.textContent = '';
        const newFiles = Array.from(files);
        
        // Evitar duplicados simples (pelo nome e tamanho)
        newFiles.forEach(file => {
            const isDuplicate = pendingFiles.some(f => f.name === file.name && f.size === file.size);
            if (!isDuplicate) {
                pendingFiles.push(file);
            }
        });

        renderFileList();
    }

    function renderFileList() {
        if (pendingFiles.length === 0) {
            dropzoneDefault.classList.remove('hidden');
            dropzonePreview.classList.add('hidden');
            compressBtn.classList.add('hidden');
            return;
        }

        dropzoneDefault.classList.add('hidden');
        dropzonePreview.classList.remove('hidden');
        compressBtn.classList.remove('hidden');
        
        selectedFilesList.innerHTML = '';
        pendingFiles.forEach((file, index) => {
            const ext = getExtension(file.name);
            const icon = fileIcons[ext] || fileIcons.default;
            
            const item = document.createElement('div');
            item.className = 'file-list-item';
            item.innerHTML = `
                <div class="file-ext-icon">${icon}</div>
                <div class="file-info">
                    <div class="file-name">${file.name}</div>
                    <div class="file-size">${formatSize(file.size)}</div>
                </div>
                <button class="remove-file-btn" data-index="${index}">
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <line x1="18" y1="6" x2="6" y2="18"></line>
                        <line x1="6" y1="6" x2="18" y2="18"></line>
                    </svg>
                </button>
            `;
            
            item.querySelector('.remove-file-btn').addEventListener('click', (e) => {
                e.stopPropagation();
                pendingFiles.splice(index, 1);
                renderFileList();
            });
            
            selectedFilesList.appendChild(item);
        });
    }

    function getExtension(fileName) {
        return fileName.split('.').pop().toLowerCase();
    }

    function formatSize(bytes) {
        if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
        return (bytes / (1024 * 1024)).toFixed(2) + ' MB';
    }

    // --- Botão Comprimir ---
    compressBtn.addEventListener('click', () => {
        if (pendingFiles.length === 0) return;
        uploadFiles();
    });

    async function uploadFiles() {
        compressBtn.disabled = true;
        compressBtn.textContent = 'Enviando...';
        toggleUI(true);

        showSection(statusSection);
        batchStatusTitle.textContent = 'Enviando arquivos...';
        batchProgressText.textContent = `0 de ${pendingFiles.length} concluídos`;
        batchProgressBar.style.width = '0%';
        
        jobsList.innerHTML = '';
        activeJobs = {};

        const formData = new FormData();
        pendingFiles.forEach(file => formData.append('file', file));
        formData.append('compression_level', compressionLevel.toString());

        try {
            const response = await fetch('/api/compress', {
                method: 'POST',
                body: formData
            });

            if (!response.ok) {
                const err = await response.json();
                throw new Error(err.error || 'Erro ao iniciar compressão');
            }

            const data = await response.json();
            const jobIds = data.ids;

            batchStatusTitle.textContent = 'Comprimindo arquivos...';
            
            // Inicializar a lista de jobs na UI
            jobIds.forEach((id, index) => {
                const file = pendingFiles[index];
                const jobElement = createJobElement(id, file.name);
                jobsList.appendChild(jobElement);
                
                activeJobs[id] = {
                    filename: file.name,
                    status: 'processing',
                    progress: 0,
                    element: jobElement
                };
                
                pollStatus(id);
            });

        } catch (e) {
            handleError(e.message);
        }
    }

    function createJobElement(id, filename) {
        const div = document.createElement('div');
        div.className = 'job-item processing';
        div.id = `job-${id}`;
        div.innerHTML = `
            <div class="job-item-header">
                <span class="job-item-name" title="${filename}">${filename}</span>
                <span class="job-status-tag">Processando</span>
            </div>
            <div class="job-mini-progress">
                <div class="job-mini-progress-fill" id="progress-fill-${id}"></div>
            </div>
            <div class="job-footer" id="footer-${id}"></div>
        `;
        return div;
    }

    async function pollStatus(jobId) {
        try {
            const res = await fetch(`/api/status/${jobId}`);
            if (!res.ok) return;

            const job = await res.json();
            const jobData = activeJobs[jobId];
            if (!jobData) return;

            // Atualizar progresso individual
            if (job.progress !== null && job.progress !== undefined) {
                jobData.progress = job.progress;
                const fill = document.getElementById(`progress-fill-${jobId}`);
                if (fill) fill.style.width = `${job.progress}%`;
            }

            if (job.status === 'completed') {
                updateJobUI(jobId, 'completed', job.filename);
            } else if (job.status === 'error') {
                updateJobUI(jobId, 'error', null, job.error);
            } else {
                setTimeout(() => pollStatus(jobId), 1000);
            }

            updateBatchProgress();

        } catch (e) {
            console.error('Erro polling job:', jobId, e);
        }
    }

    function updateJobUI(jobId, status, finalName, errorMsg = '') {
        const jobData = activeJobs[jobId];
        if (!jobData) return;
        
        jobData.status = status;
        const el = jobData.element;
        el.className = `job-item ${status}`;
        
        const tag = el.querySelector('.job-status-tag');
        const footer = el.querySelector('.job-footer');
        
        if (status === 'completed') {
            tag.textContent = 'Concluído';
            footer.innerHTML = `
                <a href="/api/download/${jobId}" class="job-download-btn" download="${finalName}">
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
                        <polyline points="7 10 12 15 17 10"></polyline>
                        <line x1="12" y1="15" x2="12" y2="3"></line>
                    </svg>
                    Baixar
                </a>
            `;
            const fill = document.getElementById(`progress-fill-${jobId}`);
            if (fill) fill.style.width = '100%';
        } else if (status === 'error') {
            tag.textContent = 'Erro';
            footer.innerHTML = `<div class="job-error-text">${humanizeError(errorMsg)}</div>`;
        }
    }

    function updateBatchProgress() {
        const jobs = Object.values(activeJobs);
        const total = jobs.length;
        const completed = jobs.filter(j => j.status === 'completed' || j.status === 'error').length;
        
        const percent = (completed / total) * 100;
        batchProgressBar.style.width = `${percent}%`;
        batchProgressText.textContent = `${completed} de ${total} concluídos`;
        
        if (completed === total) {
            batchStatusTitle.textContent = 'Processamento finalizado!';
            batchActions.classList.remove('hidden');
        }
    }

    function humanizeError(raw) {
        if (!raw) return 'Erro desconhecido.';
        if (raw.includes('No such file')) return 'Arquivo não encontrado.';
        if (raw.includes('Invalid data')) return 'Arquivo corrompido.';
        return 'Falha na compressão.';
    }

    function handleError(msg) {
        showSection(uploadSection);
        errorMsg.textContent = `⚠ ${msg}`;
        compressBtn.disabled = false;
        compressBtn.textContent = 'Comprimir Todos';
        toggleUI(false);
    }

    function showSection(section) {
        [uploadSection, statusSection].forEach(el => el.classList.add('hidden'));
        section.classList.remove('hidden');
    }

    function toggleUI(disabled) {
        const compSection = document.querySelector('.compression-section');
        if (compSection) {
            compSection.style.pointerEvents = disabled ? 'none' : 'auto';
            compSection.style.opacity = disabled ? '0.4' : '1';
        }
        pillBtns.forEach(btn => btn.disabled = disabled);
    }

    resetBtn.addEventListener('click', () => {
        pendingFiles = [];
        activeJobs = {};
        renderFileList();
        toggleUI(false);
        showSection(uploadSection);
        batchActions.classList.add('hidden');
        compressBtn.disabled = false;
        compressBtn.textContent = 'Comprimir Todos';
    });
});
