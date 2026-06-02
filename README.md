# 🌊 RotavaFlow

**RotavaFlow** é uma aplicação web estática/offline-first para edição P&ID e simulação de escoamento no navegador. Neste momento, o projeto **não precisa de backend Rust**: as telas são HTML, o estado do projeto fica no browser/arquivo `.rfm`, e o cálculo é carregado pelo módulo WASM em `static/wasm`.

## Resposta curta: para que servia o Rust?

O Rust que existia aqui fazia basicamente duas coisas:

1. Servia HTML/templates e arquivos estáticos.
2. Criava endpoints experimentais de API em memória.

Como o app já possui `index.html`, páginas em `pages/`, assets em `static/` e WASM client-side, essa camada não era necessária para o uso atual. Um servidor de arquivos simples já resolve.

## Estrutura atual

```text
.
├── index.html              # Tela inicial
├── pages/                  # Telas estáticas do app
├── static/
│   ├── js/                 # Bibliotecas e wrapper do WASM
│   ├── manifest.json       # Manifesto PWA
│   └── wasm/               # Solver WASM/JS
├── sw.js                   # Service Worker na raiz para controlar o app todo
└── *.rfm                   # Exemplos/arquivos de malha
```

## Como rodar localmente

Use qualquer servidor estático apontando para a raiz do repositório:

```bash
python3 -m http.server 8000
```

Depois abra:

- <http://127.0.0.1:8000/index.html>
- ou simplesmente <http://127.0.0.1:8000/>

Também funciona com Nginx, Caddy, Apache, `serve`, GitHub Pages ou qualquer hospedagem estática. Evite abrir direto via `file://`, porque Service Worker e WASM funcionam de forma mais previsível via HTTP.

## O que foi removido

- `Cargo.toml` e `Cargo.lock`.
- `src/main.rs`.
- `templates/` Askama.
- Roadmap/API experimental que dependia do servidor Rust.
- `static/sw.js`, porque Service Worker precisa ficar na raiz (`sw.js`) para controlar todo o app.

## O que ainda foi ajustado

- Registro do PWA centralizado em `static/js/pwa.js`, em vez de copiar o mesmo script em cada página.
- Cache offline atualizado para estratégia stale-while-revalidate: abre rápido pelo cache e atualiza em segundo plano quando houver rede.
- O editor P&ID deixou de tentar carregar `static/js/fluxo.js`, arquivo que não existe; a execução WASM fica na tela de simulação via `static/js/wrapper.js`.

## Por que `sw.js` fica na raiz?

Service Workers só controlam páginas dentro do próprio escopo. Colocando `sw.js` na raiz, ele consegue cachear e atender offline:

- `index.html`
- `pages/*.html`
- `static/js/*`
- `static/wasm/*`
- `static/manifest.json`

## Quando faria sentido trazer Rust de volta?

Rust voltaria a fazer sentido se você quiser:

- salvar projetos em banco/arquivo no servidor;
- autenticação e múltiplos usuários;
- filas de simulação pesadas no backend;
- relatórios persistidos;
- executar um solver nativo no servidor em vez de WASM no browser.

Enquanto a proposta for “abrir a interface, editar malha, salvar `.rfm` e rodar WASM no navegador”, estático é mais simples e mais fácil de manter.
