// WASM module will be loaded from pkg/lorenz.js after wasm-pack build
// For now, we'll use a mock implementation that demonstrates the UI

let wasmModule = null;

// Try to load WASM module
async function initWasm() {
    try {
        const module = await import('../pkg/lorenz.js');
        await module.default();
        wasmModule = module;
        console.log('Lorenz WASM module loaded successfully');
        updateStatus('Ready - WASM loaded');
    } catch (error) {
        console.warn('WASM module not found, using demo mode:', error.message);
        updateStatus('Demo mode - Run wasm-pack build to enable WASM');
    }
}

function updateStatus(message) {
    const outputBox = document.getElementById('outputBox');
    if (outputBox.querySelector('.label-debug')) {
        outputBox.innerHTML = `<span class="output-label label-debug">${message}</span>`;
    }
}

// Mock execution for demo mode
function mockExecute(code) {
    // Simulate execution delay
    return new Promise((resolve) => {
        setTimeout(() => {
            // Parse some basic values from the code for demo
            const meanMatch = code.match(/chaotic\(([\d.]+)/);
            const mean = meanMatch ? parseFloat(meanMatch[1]) : 100.0;
            
            // Simulate variance growth
            const propagateMatch = code.match(/propagate\([^,]+,\s*([\d.]+)/);
            const timeStep = propagateMatch ? parseFloat(propagateMatch[1]) : 1.0;
            const variance = 0.1 * Math.exp(2 * 0.1 * timeStep);
            const output = mean + (Math.random() - 0.5) * Math.sqrt(variance) * 2;
            
            resolve({
                success: true,
                output: output.toFixed(6),
                error: null,
                debug_state: `Mean: ${mean.toFixed(3)}, Variance: ${variance.toFixed(3)}, StdDev: ${Math.sqrt(variance).toFixed(3)}`
            });
        }, 500);
    });
}

// Main execution function
window.runCode = async function() {
    const code = document.getElementById('codeEditor').value;
    const runBtn = document.getElementById('runBtn');
    const outputBox = document.getElementById('outputBox');
    
    // Disable button and show loading
    runBtn.disabled = true;
    runBtn.innerHTML = '<span class="loading"></span> Running...';
    
    // Clear previous output
    outputBox.className = 'output-box';
    outputBox.innerHTML = '<span class="output-label label-debug">Executing...</span>';
    
    try {
        let result;
        
        if (wasmModule) {
            // Use WASM module
            const resultJson = wasmModule.execute_lorenz(code);
            result = typeof resultJson === 'string' ? JSON.parse(resultJson) : resultJson;
        } else {
            // Use mock execution
            result = await mockExecute(code);
        }
        
        // Display result
        if (result.success) {
            outputBox.className = 'output-box output-success';
            let html = `<span class="output-label label-success">Output</span>\n`;
            html += `<span style="color: #3fb950; font-size: 1.2rem;">${result.output}</span>`;
            
            if (result.debug_state) {
                html += `\n\n<span class="output-label label-debug">Lorenz State</span>\n`;
                html += `<span style="color: #58a6ff;">${result.debug_state}</span>`;
            }
            
            outputBox.innerHTML = html;
        } else {
            outputBox.className = 'output-box output-error';
            outputBox.innerHTML = `<span class="output-label label-error">Error</span>\n<span style="color: #f85149;">${result.error}</span>`;
        }
    } catch (error) {
        outputBox.className = 'output-box output-error';
        outputBox.innerHTML = `<span class="output-label label-error">Runtime Error</span>\n<span style="color: #f85149;">${error.message}</span>`;
    } finally {
        // Re-enable button
        runBtn.disabled = false;
        runBtn.innerHTML = 'Run Lorenz';
    }
};

// Keyboard shortcut: Ctrl+Enter to run
document.getElementById('codeEditor').addEventListener('keydown', function(e) {
    if (e.ctrlKey && e.key === 'Enter') {
        e.preventDefault();
        window.runCode();
    }
});

// Initialize WASM on page load
initWasm();