# Roadmap do servidor RotavaFlow

O servidor agora saiu do estágio de páginas estáticas e ganhou uma superfície mínima de API para apoiar um software de simulação de escoamento. A ideia é manter o avanço em camadas pequenas, testáveis e fáceis de trocar quando o solver definitivo entrar.

## 1. Saúde e diagnóstico — implementado

- Endpoint `/health` retorna nome do serviço, status, versão, schema e módulos principais.
- Endpoint `/metrics` retorna uptime e contadores de projetos, validações e simulações.
- Logs simples no terminal registram salvamento, validação e execução.

## 2. API de projeto — implementado como armazenamento em memória

- `POST /api/projects` cria projetos JSON versionados.
- `GET /api/projects` lista resumos dos projetos salvos.
- `GET /api/projects/:id` carrega um projeto.
- `PUT /api/projects/:id` atualiza ou cria um projeto com ID conhecido.
- `POST /api/projects/:id/validate` valida topologia, referências e dimensões.

## 3. Execução de simulação — implementado como solver simplificado

- `POST /api/simulations` aceita `project_id` ou um `project` inline.
- O relatório registra status atual e histórico com `queued`, `running`, `completed` ou `failed`.
- O cálculo atual estima vazão a partir de comprimento total, diâmetro médio e pressão diferencial, servindo como contrato para substituir pelo solver real.

## 4. Observabilidade técnica — implementado de forma inicial

- Métricas de projetos salvos, validações, simulações iniciadas, concluídas e falhas.
- Registro de tempo da última simulação.
- Relatório de simulação inclui versão do solver, residual, iterações, erros de validação e resultados agregados.

## 5. Próxima evolução recomendada

- Persistência local para projetos e relatórios.
- Testes automatizados de contrato da API.
- Integração do endpoint de simulação com o módulo WASM/nativo definitivo.
- Métricas em formato Prometheus quando houver necessidade de monitoramento externo.
