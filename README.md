# 🌊 RotavaFlow

**RotavaFlow** é um servidor Rust + PWA para modelagem visual e simulação de escoamento em malhas com nós, tubos, equipamentos e condições de contorno. A aplicação entrega telas HTML via Askama, assets estáticos embutidos no binário e um módulo WASM para evolução do solver no navegador.

## ✨ O que já vem pronto

- 🧭 Interface web offline-first com Service Worker e manifesto PWA.
- 🧩 Templates para editor P&ID, nós, tubos, equipamentos, simulação e relatórios.
- 📦 Assets embutidos no executável com `rust-embed`.
- 🩺 Endpoint `/health` para diagnóstico do serviço.
- 📊 Endpoint `/metrics` com contadores operacionais simples.
- 🗂️ API em memória para criar, listar, carregar, atualizar e validar projetos.
- ⚙️ API de simulação com histórico de estados `queued`, `running`, `completed` e `failed`.

## 🚀 Como rodar

```bash
cargo run
```

Depois acesse:

- App: <http://127.0.0.1:8000>
- Saúde: <http://127.0.0.1:8000/health>
- Métricas: <http://127.0.0.1:8000/metrics>

## 🧪 Testes rápidos

```bash
cargo test
```

Criar um projeto de exemplo:

```bash
curl -fsS http://127.0.0.1:8000/api/projects \
  -H 'Content-Type: application/json' \
  -d '{
    "name": "Malha demo",
    "nodes": [
      {"id": "n1", "label": "Entrada", "x": 0, "y": 0},
      {"id": "n2", "label": "Saída", "x": 200, "y": 0}
    ],
    "ducts": [
      {"id": "t1", "from": "n1", "to": "n2", "length_m": 12.5, "diameter_m": 0.2, "roughness_m": 0.0001}
    ],
    "equipments": [
      {"id": "b1", "kind": "pump", "node_id": "n1", "pressure_delta_pa": 150000}
    ],
    "boundary_conditions": [
      {"node_id": "n1", "kind": "pressure", "value": 101325, "unit": "Pa"},
      {"node_id": "n2", "kind": "flow", "value": 0.2, "unit": "m3/s"}
    ]
  }'
```

Rodar uma simulação usando o projeto salvo:

```bash
curl -fsS http://127.0.0.1:8000/api/simulations \
  -H 'Content-Type: application/json' \
  -d '{"project_id":"proj-1"}'
```

## 🧱 Formato do projeto

O schema atual é `1.0` e modela:

| Campo | Descrição |
| --- | --- |
| `name` | Nome humano do projeto. |
| `schema_version` | Versão do contrato JSON; assume `1.0` quando omitido. |
| `nodes` | Pontos da malha hidráulica. |
| `ducts` | Tubos conectando nós, com comprimento, diâmetro e rugosidade. |
| `equipments` | Bombas, válvulas ou outros componentes presos a nós. |
| `boundary_conditions` | Pressão, vazão ou outras condições aplicadas aos nós. |

## 🔌 Endpoints principais

| Método | Rota | Uso |
| --- | --- | --- |
| `GET` | `/health` | Confirma status, versão e módulos do servidor. |
| `GET` | `/metrics` | Retorna uptime e contadores de projetos/simulações. |
| `GET` | `/api/projects` | Lista projetos salvos em memória. |
| `POST` | `/api/projects` | Cria um projeto após validação estrutural. |
| `GET` | `/api/projects/:id` | Carrega um projeto salvo. |
| `PUT` | `/api/projects/:id` | Atualiza ou cria um projeto com ID definido. |
| `POST` | `/api/projects/:id/validate` | Revalida um projeto salvo. |
| `POST` | `/api/simulations` | Executa uma simulação simplificada por `project_id` ou `project` inline. |
| `GET` | `/api/simulations/:id` | Recupera o relatório de uma simulação. |

## 🛣️ Próximos passos sugeridos

1. Persistir projetos em arquivo ou banco local em vez de manter apenas em memória.
2. Conectar `/api/simulations` ao solver WASM/nativo real.
3. Adicionar autenticação opcional quando o servidor sair do uso local.
4. Expor métricas em formato Prometheus, se houver implantação em cluster.
5. Criar testes de contrato para impedir regressões no schema JSON.

## 🧰 Stack

- Rust 2021
- Axum + Tokio
- Askama
- Rust Embed
- Serde
