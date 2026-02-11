// static/js/wrapper.js

let _mod = null;
let _fnFluxo = null;
let _fnDt = null;

export async function initFluxoWasm() {
    if (_mod) return _mod;
    const dynamicImport = new Function('path', 'return import(path)');
    const fluxoUrl = '/static/wasm/fluxo.js';
    const ModuleFactory = (await dynamicImport(fluxoUrl)).default;
    _mod = await ModuleFactory({
        locateFile: (path) => path.endsWith('.wasm') ? '/static/wasm/fluxo.wasm' : path
    });
    _fnFluxo = _mod.cwrap('Fluxo_TS_WASM', 'number', Array(13).fill('number'));
    _fnDt = _mod.cwrap('dt_2024_01_WASM', 'number', Array(13).fill('number'));
    console.info('[fluxo-wasm] inicializado');
    return _mod;
}

// Auxiliar de Auditoria
function compararVetores(nome, atual, gabarito) {
    if (!gabarito) return;
    const atualArr = Array.from(atual);
    if (atualArr.length !== gabarito.length) {
        console.error(`[Diferença] ${nome}: Tamanhos diferentes! Atual: ${atualArr.length}, Gabarito: ${gabarito.length}`);
        return;
    }
    let erros = 0;
    for (let i = 0; i < atualArr.length; i++) {
        const diff = Math.abs(atualArr[i] - gabarito[i]);
        const tol = (nome.includes("ASCII") || nome.includes("Conectividade")) ? 0 : 1e-6;
        if (diff > tol) {
            if (erros < 5) console.warn(`[Diferença] ${nome} [${i}]: Gerado=${atualArr[i]}, Esperado=${gabarito[i]}`);
            erros++;
        }
    }
    if (erros === 0) console.log(`✅ [OK] ${nome} coincide perfeitamente.`);
    else console.error(`❌ [ERRO] ${nome} tem ${erros} divergentes.`);
}

function flattenColMajor(rows, cols, getter, isInt = false) {
    const buf = isInt ? new Int32Array(rows * cols) : new Float64Array(rows * cols);
    let k = 0;
    for (let j = 0; j < cols; j++) for (let i = 0; i < rows; i++) buf[k++] = getter(i, j);
    return buf;
}

function mallocCopy(mod, typed) {
    const ptr = mod._malloc(typed.byteLength);
    mod.HEAPU8.set(new Uint8Array(typed.buffer, typed.byteOffset, typed.byteLength), ptr);
    return ptr;
}

export async function runFluxo(built, which = 'Fluxo_TS_WASM') {
    let gabarito = null;
    try {
        const resp = await fetch('/static/GABARITO_Fluxo_TS_WASM_1770834472206.json');
        gabarito = await resp.json();
    } catch (e) {}

    const mod = await initFluxoWasm();
    const n_dutos = built.dutos_in.length, n_trechos = built.geometria_in.length, n_nos = built.tipos_nos_in.length;
    const n_equip = (built.tipos_equip_in || []).length, ci_rows = n_trechos + n_dutos;

    const dutos_i32 = flattenColMajor(n_dutos, 3, (i, j) => built.dutos_in[i][j], true);
    const geom_f64 = flattenColMajor(n_trechos, 7, (i, j) => built.geometria_in[i][j]);
    const tipos_i32 = Int32Array.from(built.tipos_nos_in);
    const nos_f64 = flattenColMajor(n_nos, 2, (i, j) => built.nos_in[i][j]);
    const comp_f64 = flattenColMajor(n_nos, 14, (i, j) => built.composicao_nos_in[i][j]);
    const ci_f64 = flattenColMajor(ci_rows, 17, (i, j) => built.cond_inic_in[i][j]);
    const num_f64 = Float64Array.from(built.numerico || []);

    if (gabarito) {
        console.group("🔍 Auditoria Pre-WASM");
        compararVetores("Dutos (Conectividade)", dutos_i32, gabarito.INTEIROS.dutos_conectividade);
        compararVetores("Tipos Nós (ASCII)", tipos_i32, gabarito.INTEIROS.tipos_nos_ascii);
        compararVetores("Geometria (7 colunas)", geom_f64, gabarito.DECIMAIS.geometria);
        compararVetores("Nós Valores (P/Q/T)", nos_f64, gabarito.DECIMAIS.nos_valores);
        compararVetores("Composição (14 componentes)", comp_f64, gabarito.DECIMAIS.composicao);
        compararVetores("Condição Inicial", ci_f64, gabarito.DECIMAIS.cond_inic);
        compararVetores("Parâmetros Numéricos", num_f64, gabarito.DECIMAIS.numerico);
        console.groupEnd();
    }

    const p_ci = mallocCopy(mod, ci_f64), p_geom = mallocCopy(mod, geom_f64), p_dutos = mallocCopy(mod, dutos_i32);
    const p_comp = mallocCopy(mod, comp_f64), p_tnos = mallocCopy(mod, tipos_i32), p_nos = mallocCopy(mod, nos_f64);
    const p_te = n_equip ? mallocCopy(mod, Int32Array.from(built.tipos_equip_in)) : 0;
    const p_equip = n_equip ? mallocCopy(mod, flattenColMajor(n_equip, 2, (i, j) => built.equip_in[i][j], true)) : 0;
    const p_ve = n_equip ? mallocCopy(mod, Float64Array.from(built.val_equip)) : 0;
    const p_num = mallocCopy(mod, num_f64);

    try {
        const fn = (which === 'dt_2024_01_WASM') ? _fnDt : _fnFluxo;
        const ret = fn(p_ci, p_geom, n_dutos, p_dutos, p_comp, n_nos, p_tnos, p_nos, n_equip, p_te, p_equip, p_ve, p_num);
        const base = p_ci >> 3;
        const out = Array.from({ length: ci_rows }, (_, i) => Array.from({ length: 17 }, (_, j) => mod.HEAPF64[base + (i + ci_rows * j)]));
        return { ret, cond_inic_in: out };
    } finally {
        [p_ci, p_geom, p_dutos, p_comp, p_tnos, p_nos, p_te, p_equip, p_ve, p_num].forEach(p => p && mod._free(p));
    }
}