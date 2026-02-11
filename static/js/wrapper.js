// src/wasm/wrapper.js
// Wrapper para rodar fluxo.wasm/fluxo.js que estão no /public

let _mod = null
let _fnFluxo = null
let _fnDt = null

export async function initFluxoWasm() {
  if (_mod) return _mod

  // cria import dinâmico fora do alcance do vite
  const dynamicImport = new Function('path', 'return import(path)')

  // monta string dinamicamente pra não ser analisada
  const fluxoUrl = '/' + 'fluxo.js'
  const ModuleFactory = (await dynamicImport(fluxoUrl)).default

  _mod = await ModuleFactory({
    locateFile: (path) => {
      if (path.endsWith('.wasm')) return '/fluxo.wasm'
      return path
    }
  })

  _fnFluxo = _mod.cwrap('Fluxo_TS_WASM', 'number',
    Array(13).fill('number'))
  _fnDt = _mod.cwrap('dt_2024_01_WASM', 'number',
    Array(13).fill('number'))

  console.info('[fluxo-wasm] inicializado')
  return _mod
}

// ------------ helpers ------------
function flattenColMajor(rows, cols, getter, isInt=false) {
  const buf = isInt ? new Int32Array(rows*cols) : new Float64Array(rows*cols)
  let k = 0
  for (let j=0;j<cols;j++) for (let i=0;i<rows;i++) buf[k++] = getter(i,j)
  return buf
}
function toInt32(arr){return Int32Array.from(arr||[])}
function toF64(arr){return arr instanceof Float64Array? arr : Float64Array.from(arr||[])}

function mallocCopy(mod, typed){
  const ptr = mod._malloc(typed.byteLength)
  mod.HEAPU8.set(new Uint8Array(typed.buffer, typed.byteOffset, typed.byteLength), ptr)
  return ptr
}
function freeAll(mod, ...ptrs){ ptrs.forEach(p=>p&&mod._free(p)) }

function expandColMajor2D_F64(mod, ptr, rows, cols) {
  const out = Array.from({length:rows},()=>new Array(cols).fill(0))
  const base = ptr>>3
  for (let j=0;j<cols;j++) for (let i=0;i<rows;i++){
    out[i][j] = mod.HEAPF64[base + (i + rows*j)]
  }
  return out
}

// ------------ chamada de alto nível ------------
export async function runFluxo(built, which='Fluxo_TS_WASM'){
  const mod = await initFluxoWasm()

  const n_dutos   = built.dutos_in.length
  const n_trechos = built.geometria_in.length
  const n_nos     = built.tipos_nos_in.length
  const n_equip   = (built.tipos_equip_in||[]).length
  const ci_rows   = n_trechos + n_dutos

  const dutos_i32   = flattenColMajor(n_dutos,3,(i,j)=>built.dutos_in[i][j],true)
  const geom_f64    = flattenColMajor(n_trechos,7,(i,j)=>built.geometria_in[i][j])
  const tipos_i32   = toInt32(built.tipos_nos_in)
  const nos_f64     = flattenColMajor(n_nos,2,(i,j)=>built.nos_in[i][j])
  const comp_f64    = flattenColMajor(n_nos,14,(i,j)=>built.composicao_nos_in[i][j])
  const te_i32      = toInt32(built.tipos_equip_in||[])
  const equip_i32   = n_equip ? flattenColMajor(n_equip,2,(i,j)=>built.equip_in[i][j],true) : new Int32Array(0)
  const ve_f64      = toF64(built.val_equip||[])
  const numerico_f64= toF64(built.numerico||[])
  const ci_f64      = flattenColMajor(ci_rows,17,(i,j)=>built.cond_inic_in[i][j])

  const p_ci    = mallocCopy(mod, ci_f64)
  const p_geom  = mallocCopy(mod, geom_f64)
  const p_dutos = mallocCopy(mod, dutos_i32)
  const p_comp  = mallocCopy(mod, comp_f64)
  const p_tnos  = mallocCopy(mod, tipos_i32)
  const p_nos   = mallocCopy(mod, nos_f64)
  const p_te    = n_equip ? mallocCopy(mod, te_i32) : 0
  const p_equip = n_equip ? mallocCopy(mod, equip_i32) : 0
  const p_ve    = n_equip ? mallocCopy(mod, ve_f64) : 0
  const p_num   = mallocCopy(mod, numerico_f64)

  try{
    const fn = which==='dt_2024_01_WASM'? _fnDt : _fnFluxo
    const ret = fn(p_ci,p_geom,n_dutos,p_dutos,p_comp,n_nos,p_tnos,p_nos,n_equip,p_te,p_equip,p_ve,p_num)
    const cond_inic_in = expandColMajor2D_F64(mod,p_ci,ci_rows,17)
    return {ret, cond_inic_in}
  }finally{
    freeAll(mod,p_ci,p_geom,p_dutos,p_comp,p_tnos,p_nos,p_te,p_equip,p_ve,p_num)
  }
}
