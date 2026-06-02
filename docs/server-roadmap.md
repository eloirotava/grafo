# Sugestões iniciais para o servidor RotavaFlow

Este servidor já entrega uma PWA simples com templates Askama, assets embutidos e o módulo WASM. Para um software de simulação de escoamento, eu começaria evoluindo em camadas pequenas e verificáveis.

## 1. Saúde e diagnóstico

- Expor um endpoint `/health` para confirmar que o servidor está online, qual versão está rodando e quais módulos principais estão disponíveis.
- Usar esse endpoint em scripts de inicialização, testes automatizados e telas futuras de suporte.

## 2. API de projeto

- Criar endpoints para salvar, carregar e validar projetos de malha.
- Definir um formato JSON versionado para nós, tubos, equipamentos, condições de contorno e metadados do modelo.
- Validar entradas antes de enviar dados ao WASM, evitando simulações com unidades incompatíveis ou topologia incompleta.

## 3. Execução de simulação

- Separar a execução em uma rota de API dedicada, por exemplo `/api/simulations`.
- Retornar estados de simulação como `queued`, `running`, `completed` e `failed`.
- Armazenar erros numéricos e mensagens de convergência de forma estruturada para alimentar a tela de relatórios.

## 4. Observabilidade técnica

- Adicionar logs com tempo de execução, tamanho da malha, número de iterações e erro residual final.
- Criar métricas simples para detectar regressões de desempenho.
- Guardar relatórios de simulação com versão do solver e parâmetros usados.

## 5. Caminho recomendado

A primeira melhoria implementada nesta branch é o endpoint `/health`, porque ele é pequeno, fácil de testar e cria uma base para automação e suporte operacional.
