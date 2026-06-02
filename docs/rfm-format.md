# Formato `.rfm` do RotavaFlow

O arquivo `.rfm` é um JSON do projeto compactado com gzip quando salvo pelo botão **Salvar Malha**. Para depuração, o Editor P&ID também permite exportar o mesmo conteúdo como JSON legível.

## Metadados

| Campo | Tipo | Descrição |
| --- | --- | --- |
| `schema_version` | string | Versão do contrato de dados. A versão atual é `1.0`. |
| `version` | string | Versão da aplicação/formato visual que gerou o arquivo. |
| `timestamp` | number | Data de salvamento em milissegundos desde Unix epoch. |
| `updated_at` | string | Data ISO do salvamento/exportação. |

## Entidades principais

| Campo | Descrição |
| --- | --- |
| `nodes` | Nós desenhados no canvas, com posição, tipo visual e tipo físico original. |
| `ducts` | Dutos conectando nós, com ids das pontas e portas de conexão. |
| `valves` | Válvulas/equipamentos anexados a dutos. |
| `duct_geom` | Tramos geométricos dos dutos: comprimento, desnível, diâmetro, rugosidade, troca térmica e temperatura ambiente. |
| `node_props` | Condições físicas dos nós: pressão, vazão, temperatura, volume e elevação. |
| `node_composition` | Frações molares por componente para cada nó. |

## Validação antes de salvar/simular

A validação atual verifica:

- pelo menos dois nós e um duto;
- dutos conectados nas duas pontas;
- referências de dutos para nós existentes;
- ids duplicados de nós ou dutos;
- comprimento e diâmetro positivos nos tramos;
- rugosidade não negativa;
- aviso para nós sem composição;
- aviso para dutos sem geometria detalhada.

Erros bloqueiam salvamento e simulação. Avisos permitem continuar, mas indicam que o solver pode usar valores padrão ou dados incompletos.

## Exemplo mínimo

```json
{
  "schema_version": "1.0",
  "version": "2.1",
  "timestamp": 1780420000000,
  "updated_at": "2026-06-02T18:00:00.000Z",
  "nodes": [
    { "id": "N1", "name": "Entrada", "x": 120, "y": 100, "type": "node", "originalType": "P" },
    { "id": "N2", "name": "Saída", "x": 420, "y": 100, "type": "node", "originalType": "Q" }
  ],
  "ducts": [
    { "id": "D1", "name": "Duto 1", "start_id": "N1", "end_id": "N2", "start_port": 0, "end_port": 0 }
  ],
  "valves": [],
  "duct_geom": [
    { "duct_id": "D1", "seg_index": 0, "L": 100, "dy": 0, "D": 0.3937, "rug": 0.00003, "U": 50, "aco": "0", "Tamb": 298 }
  ],
  "node_props": [
    { "node_id": "N1", "pressure": 50, "temperature": 300 },
    { "node_id": "N2", "flow": 1.2, "temperature": 300 }
  ],
  "node_composition": [
    { "node_id": "N1", "component": "C1", "value": 0.9, "fraction": 0.9 },
    { "node_id": "N1", "component": "CO2", "value": 0.1, "fraction": 0.1 }
  ]
}
```
