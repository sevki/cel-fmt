// Import the WASM module (this will be available after build)
import init, { format_with_options, version } from './pkg/cel_fmt.js';

const inputEl = document.getElementById('input');
const outputEl = document.getElementById('output');
const formatBtn = document.getElementById('formatBtn');
const clearBtn = document.getElementById('clearBtn');
const copyBtn = document.getElementById('copyBtn');
const errorEl = document.getElementById('error');
const loadingEl = document.getElementById('loading');
const versionEl = document.getElementById('version');

// Configuration inputs
const maxWidthInput = document.getElementById('maxWidth');
const indentWidthInput = document.getElementById('indentWidth');
const useTabsInput = document.getElementById('useTabs');
const trailingCommaInput = document.getElementById('trailingComma');

let wasmReady = false;

// Initialize WASM
async function initWasm() {
    try {
        loadingEl.classList.remove('hidden');
        await init();
        wasmReady = true;
        loadingEl.classList.add('hidden');

        // Set version
        versionEl.textContent = version();

        // Enable format button
        formatBtn.disabled = false;

        // Auto-format initial content
        formatCode();
    } catch (err) {
        showError(`Failed to load formatter: ${err.message}`);
        loadingEl.classList.add('hidden');
    }
}

// Format the code
function formatCode() {
    if (!wasmReady) {
        showError('Formatter is still loading, please wait...');
        return;
    }

    const source = inputEl.value.trim();
    if (!source) {
        outputEl.value = '';
        hideError();
        return;
    }

    try {
        const maxWidth = parseInt(maxWidthInput.value) || 80;
        const indentWidth = parseInt(indentWidthInput.value) || 2;
        const useTabs = useTabsInput.checked;
        const trailingComma = trailingCommaInput.checked;

        const formatted = format_with_options(
            source,
            maxWidth,
            indentWidth,
            useTabs,
            trailingComma
        );

        outputEl.value = formatted;
        hideError();
    } catch (err) {
        showError(err.toString());
        outputEl.value = '';
    }
}

// Copy formatted output to clipboard
async function copyOutput() {
    const text = outputEl.value;
    if (!text) return;

    try {
        await navigator.clipboard.writeText(text);
        const originalText = copyBtn.textContent;
        copyBtn.textContent = '✓ Copied!';
        setTimeout(() => {
            copyBtn.textContent = originalText;
        }, 2000);
    } catch (err) {
        showError('Failed to copy to clipboard');
    }
}

// Clear input
function clearInput() {
    inputEl.value = '';
    outputEl.value = '';
    hideError();
    inputEl.focus();
}

// Show error message
function showError(message) {
    errorEl.textContent = message;
    errorEl.classList.remove('hidden');
}

// Hide error message
function hideError() {
    errorEl.classList.add('hidden');
}

// Event listeners
formatBtn.addEventListener('click', formatCode);
clearBtn.addEventListener('click', clearInput);
copyBtn.addEventListener('click', copyOutput);

// Auto-format on input (debounced)
let formatTimeout;
inputEl.addEventListener('input', () => {
    clearTimeout(formatTimeout);
    formatTimeout = setTimeout(formatCode, 500);
});

// Format on config change
maxWidthInput.addEventListener('change', formatCode);
indentWidthInput.addEventListener('change', formatCode);
useTabsInput.addEventListener('change', formatCode);
trailingCommaInput.addEventListener('change', formatCode);

// Keyboard shortcuts
document.addEventListener('keydown', (e) => {
    // Ctrl/Cmd + Enter to format
    if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
        e.preventDefault();
        formatCode();
    }

    // Ctrl/Cmd + K to clear
    if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
        e.preventDefault();
        clearInput();
    }
});

// Initialize
initWasm();
