# RotavaFlow

Simulador de escoamento composicional em regime permanente e transiente, com
editor P&ID, que roda **inteiramente no navegador** — inclusive offline.

Não há servidor, backend nem etapa de build. O repositório *é* o site: um punhado
de ficheiros HTML estáticos, mais o solver compilado para WebAssembly. Nenhum dado
sai da máquina — a malha vive no `localStorage` e o cálculo corre no seu próprio
processador.

---

## Como usar

O site precisa de ser servido por HTTP. Abrir o `index.html` com duplo clique
(`file://`) **não funciona**: o navegador bloqueia o módulo do solver por CORS
(origem `null`) e o Service Worker não regista sobre o protocolo `file:`.

```bash
# qualquer servidor estático serve
python3 -m http.server 8000
# depois abra http://localhost:8000
```

Publicado, é só abrir o endereço. Pelo menu **⋮ → Instalar** o navegador instala
a aplicação no computador ou telemóvel; a partir daí abre como um programa
normal e funciona sem rede.

## Fluxo de trabalho

| Página | O que se faz |
| --- | --- |
| **Editor P&ID** | Arrastar nós, válvulas e equipamentos; ligar com dutos (as pontas fazem *snap* e ficam verdes). Duplo clique num componente salta para a página dele. |
| **Nós** | Tipo do nó — `P` (pressão), `Q` (vazão) ou `I` (volume) — temperatura e composição molar em 14 componentes (N2, CO2, C1…C10+). |
| **Tubos** | Geometria por trecho: comprimento, desnível, diâmetro, rugosidade, coeficiente de troca térmica e temperatura ambiente. |
| **Equipamentos** | Válvulas: aberta, fechada, ou com pressão/vazão imposta. |
| **Simulação** | Regime permanente ou transiente, passo de rastreio, relaxação e tolerância. Corre o solver. |
| **Relatórios** | Perfis de pressão, temperatura, vazão e composição ao longo de cada duto. |

O projeto guarda-se em ficheiro `.rfm` (**R**otava**F**low **M**esh) pelos botões
do editor — é JSON comprimido com gzip, escrito pela `CompressionStream` do
próprio navegador.

## Como funciona

**O solver** é código nativo compilado para WebAssembly com Emscripten
(`static/wasm/fluxo.wasm`, 338 KB). O `static/js/wrapper.js` faz a ponte: achata as
matrizes de entrada em ordem *column-major*, copia-as para a heap do módulo,
chama a função exportada e lê o resultado de volta. Para dar ordem de grandeza,
a malha de exemplo deste repositório — 58 dutos, 1371 trechos — converge em
cerca de 3 segundos.

**O modo offline** é um Service Worker (`sw.js`, na raiz, que é onde tem de estar
para o escopo cobrir `/pages/`). Ele pré-carrega os 18 recursos do site na
instalação e depois serve HTML pela rede primeiro, para as páginas nunca ficarem
presas numa versão antiga, e JS/CSS/WASM pelo cache primeiro, revalidando em
segundo plano — sem isso, cada carregamento esperaria pela rede por 338 KB de
WASM. Com a rede desligada, as páginas abrem e a simulação corre na mesma.

**As bibliotecas** ([Fabric.js](http://fabricjs.com/) no editor,
[Chart.js](https://www.chartjs.org/) nos relatórios) estão versionadas em
`static/js/`. Não há CDN: é um requisito do modo offline, e mantém o site
reprodutível.

## Estrutura

```
index.html            página inicial
sw.js                 service worker (tem de ficar na raiz)
pages/                as 7 páginas da aplicação
static/
  css/app.css         layout partilhado por todas as páginas
  js/app.js           registo do service worker
  js/wrapper.js       ponte JavaScript ↔ WASM
  js/                 fabric.js, chart.js
  wasm/               solver compilado
  manifest.json       metadados de PWA
*.rfm                 malhas de exemplo
```

## Publicação

O deploy para GitHub Pages é automático a cada push em `main` que toque no site
(`.github/workflows/deploy_pages.yml`), e também corre pelo botão manual em
*Actions*. O workflow monta o artefacto só com o site — as malhas de exemplo
ficam de fora.

> **Antes do primeiro deploy:** em *Settings → Pages*, defina **Source =
> GitHub Actions**. Enquanto estiver em "Deploy from a branch", o workflow falha.

Os caminhos são todos relativos, por isso o site funciona tanto na raiz de um
domínio como numa subpasta (`utilizador.github.io/grafo/`).
