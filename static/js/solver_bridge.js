// static/js/solver_bridge.js
// Ponte entre o Canvas (UI) e o Solver (C++/Wasm)

let solverModule = null;

function onSolverReady(module) {
    console.log("🔋 Solver WASM carregado!");
    solverModule = module;
}

// Por enquanto, apenas um placeholder para não quebrar o site
window.solverBridge = {
    run: function(data) {
        console.log("Simulação solicitada:", data);
        if(!solverModule) return alert("Motor de cálculo ainda carregando...");
        // Aqui conectaremos com o C++ depois
        alert("Simulação pronta para ser implementada!");
    }
};