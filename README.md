# 🌊 RotavaFlow

**RotavaFlow** é uma aplicação web offline-first para desenhar malhas P&ID, configurar propriedades de nós/dutos/equipamentos e executar simulações de escoamento diretamente no navegador com WebAssembly.

## ✨ Principais recursos

- **Editor P&ID visual:** monte a malha arrastando nós, equipamentos, válvulas e dutos no canvas.
- **Configuração física:** edite propriedades dos nós, geometria dos dutos, composições e parâmetros de equipamentos.
- **Simulação no navegador:** carregamento do solver WASM em `static/wasm` pela tela de Simulação.
- **Relatórios:** visualização dos resultados salvos da última simulação em gráficos e tabelas.
- **Projetos portáveis:** salve e carregue arquivos `.rfm` para continuar o trabalho em outra máquina.
- **PWA/offline:** o Service Worker mantém as páginas, bibliotecas e o solver em cache para uso sem internet depois do primeiro acesso.

## 🚀 Como rodar localmente

Use qualquer servidor estático apontando para a raiz do repositório:

```bash
python3 -m http.server 8000
```

Depois abra:

- <http://127.0.0.1:8000/>
- <http://127.0.0.1:8000/index.html>

> Dica: prefira rodar via HTTP local em vez de abrir o arquivo direto via `file://`, porque Service Worker e WASM funcionam de forma mais previsível em um servidor.

## 🧭 Fluxo recomendado de uso

1. Acesse **Editor P&ID** e monte a malha no canvas.
2. Dê duplo clique nos elementos ou navegue pelas abas **Nós**, **Tubos** e **Equipamentos** para preencher os parâmetros físicos.
3. Salve o projeto como `.rfm` sempre que quiser guardar uma versão da malha.
4. Abra **Simulação**, confira as opções numéricas e rode o solver.
5. Vá para **Relatórios** para revisar os resultados da última simulação.

## 📁 Estrutura do projeto

```text
.
├── index.html              # Tela inicial
├── pages/                  # Telas do editor, configuração, simulação e relatórios
├── static/
│   ├── js/                 # Bibliotecas, PWA helper e wrapper do solver
│   ├── manifest.json       # Manifesto PWA
│   └── wasm/               # Solver WebAssembly e loader gerado
├── sw.js                   # Service Worker da aplicação
└── *.rfm                   # Exemplos/arquivos de malha
```

## 📦 Arquivos importantes

| Caminho | Função |
| --- | --- |
| `index.html` | Página inicial e ponto de entrada do app. |
| `pages/canvas.html` | Editor visual da malha P&ID. |
| `pages/nodes.html` | Configuração dos nós. |
| `pages/ducts.html` | Configuração dos dutos/tubos. |
| `pages/equipments.html` | Configuração de equipamentos e válvulas. |
| `pages/simulation.html` | Montagem dos dados e chamada do solver WASM. |
| `pages/reports.html` | Leitura do último relatório salvo no navegador. |
| `static/js/wrapper.js` | Ponte JavaScript para chamar as funções exportadas pelo WASM. |
| `static/js/pwa.js` | Registro centralizado do Service Worker. |
| `sw.js` | Cache offline das páginas e assets principais. |

## 🗺️ Roadmap

As próximas melhorias planejadas estão em [`docs/roadmap.md`](docs/roadmap.md), incluindo validação da malha, documentação do `.rfm`, relatórios melhores e acabamento PWA.

## 📴 Uso offline

No primeiro acesso via servidor HTTP, o navegador instala o PWA e armazena o app shell em cache. Depois disso, o RotavaFlow consegue abrir as telas principais e carregar o solver mesmo sem rede.

Se alguma alteração não aparecer durante desenvolvimento, limpe o cache do navegador ou recarregue ignorando cache. O cache atual usa a chave `rotavaflow-static-v2`.

## 🧪 Checklist rápido para desenvolvimento

```bash
python3 -m json.tool static/manifest.json
node --check static/js/pwa.js
node --check sw.js
node --check static/js/wrapper.js
python3 -m http.server 8000
```

Com o servidor rodando, confira pelo navegador:

- `/` abre a home.
- `/pages/canvas.html` abre o editor.
- `/pages/simulation.html` carrega a tela de simulação.
- `/static/wasm/fluxo.wasm` responde pelo servidor.
