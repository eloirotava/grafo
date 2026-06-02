# Roadmap do RotavaFlow

Este documento registra o que ainda vale fazer no RotavaFlow como aplicação web offline-first. A ideia é priorizar melhorias que aumentem estabilidade, clareza dos dados e confiança na simulação sem complicar a distribuição estática.

## 1. Conteúdo e experiência do usuário

- Padronizar textos das telas para usar os mesmos nomes: **nós**, **tubos/dutos**, **equipamentos**, **simulação** e **relatórios**.
- Melhorar a página de Ajuda com exemplos curtos de um projeto completo: criar dois nós, conectar um duto, configurar propriedades e rodar a simulação.
- Adicionar avisos visuais quando não houver projeto salvo no navegador ou quando a malha estiver incompleta.
- Exibir um resumo do projeto atual no topo das páginas de configuração: quantidade de nós, dutos, válvulas/equipamentos e última alteração.

## 2. Dados e arquivo `.rfm`

- Documentar o formato do `.rfm` com campos esperados para nós, dutos, geometria, composição, equipamentos e resultados.
- Criar validação antes de salvar/simular para detectar referências quebradas, dutos sem ponta conectada, diâmetro inválido, composição incompleta e unidades ausentes.
- Incluir `schema_version` no arquivo salvo para permitir migrações futuras sem quebrar projetos antigos.
- Adicionar um botão de exportação legível em JSON além do `.rfm` comprimido, útil para depuração.

## 3. Simulação WASM

- Mostrar estado de carregamento do solver WASM com mensagens claras: carregando, pronto, executando, convergiu ou falhou.
- Transformar erros do solver em mensagens de usuário, evitando mostrar apenas exceções técnicas.
- Registrar no relatório os parâmetros numéricos usados, número de iterações, código de retorno e tempo de execução.
- Criar um caso de teste manual com uma malha pequena conhecida para conferir se os resultados continuam coerentes após mudanças.

## 4. Relatórios

- Melhorar a tela de Relatórios para mostrar cards de resumo: pressão mínima/máxima, temperatura mínima/máxima, vazão média e status de convergência.
- Permitir baixar o relatório em JSON ou CSV.
- Mostrar quando não existe `last_sim_report` no navegador e orientar o usuário a rodar a simulação primeiro.
- Adicionar comparação simples entre duas simulações salvas, se houver histórico local.

## 5. PWA e distribuição

- Adicionar ícones reais ao manifesto para instalação como aplicativo.
- Criar uma página offline amigável para quando um arquivo não estiver no cache.
- Revisar o cache sempre que novos arquivos forem adicionados, mantendo `sw.js` atualizado.
- Testar a publicação em uma hospedagem estática, como GitHub Pages, Netlify, Caddy ou Nginx.

## 6. Qualidade e manutenção

- Criar um script simples de verificação para validar manifesto, sintaxe dos JavaScripts e referências `href`/`src` dos HTMLs.
- Separar JavaScript inline grande em arquivos dentro de `static/js/`, começando por `pages/canvas.html` e `pages/simulation.html`.
- Padronizar estilos compartilhados em um CSS comum para reduzir duplicação entre páginas.
- Adicionar exemplos pequenos de `.rfm` em uma pasta `examples/` com nomes sem espaços.

## Prioridade sugerida

1. Validação da malha antes da simulação.
2. Mensagens de estado/erro do solver WASM.
3. Documentação do formato `.rfm`.
4. Melhorias da tela de Relatórios.
5. Extração de JavaScript/CSS compartilhado.
6. Ícones e acabamento PWA.
