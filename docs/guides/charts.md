---
title: Gráficos
description: Transforme uma tabela em gráfico de barras, linhas, área, pizza ou rosca com o bloco chart.
---

Na Ribalta, um gráfico é uma tabela dentro de um bloco `::: chart`. Você escreve os números como texto e a Ribalta desenha o gráfico com as cores do tema.

## Do texto ao gráfico

```ribalta preview
# Vendas por trimestre
::: chart
| Trimestre | Vendas |
| --- | ---: |
| T1 | 120 |
| T2 | 145 |
| T3 | 162 |
| T4 | 190 |
:::
```

A tabela segue uma regra simples:

- **A primeira coluna** tem os rótulos (o eixo horizontal, ou as fatias de uma pizza).
- **Cada uma das outras colunas** é uma série de valores. O cabeçalho da coluna é o nome da série.

Sem nenhum atributo, o gráfico é de barras.

## Tipos de gráfico

Escolha o tipo com o atributo `type`:

| Valor | Gráfico |
| --- | --- |
| `bar` | Barras verticais. É o padrão. |
| `line` | Linhas. |
| `area` | Linhas com a área abaixo preenchida. |
| `pie` | Pizza. |
| `donut` | Rosca (pizza com o centro vazado). |

### Barras

```ribalta preview
# Receita por região
::: chart {type=bar values}
| Região | 2025 | 2026 |
| --- | ---: | ---: |
| Norte | 32 | 41 |
| Sul | 28 | 45 |
| Leste | 40 | 52 |
:::
```

### Linhas

```ribalta preview
# Usuários ativos
::: chart {type=line}
| Mês | Web | Celular |
| --- | ---: | ---: |
| Jan | 1200 | 800 |
| Fev | 1350 | 1100 |
| Mar | 1500 | 1480 |
| Abr | 1620 | 1900 |
:::
```

### Área

```ribalta preview
# Tráfego do site
::: chart {type=area}
| Semana | Visitas |
| --- | ---: |
| S1 | 4200 |
| S2 | 5100 |
| S3 | 4800 |
| S4 | 6300 |
:::
```

### Pizza e rosca

Em `pie` e `donut`, apenas a **primeira série** é usada, e cada linha da tabela vira uma fatia:

```ribalta preview
# Origem dos clientes
::: columns {gap=xl}
::: chart {type=pie values}
| Canal | Clientes |
| --- | ---: |
| Indicação | 45 |
| Busca | 30 |
| Eventos | 25 |
:::
::: chart {type=donut values}
| Canal | Clientes |
| --- | ---: |
| Indicação | 45 |
| Busca | 30 |
| Eventos | 25 |
:::
:::
```

## Opções

| Atributo | O que faz |
| --- | --- |
| `type=bar` | Tipo do gráfico: `bar`, `line`, `area`, `pie` ou `donut`. |
| `values` | Mostra o valor de cada ponto ou barra. Em `pie` e `donut`, mostra a porcentagem de cada fatia. |
| `legend` | Mostra a legenda. Por padrão, ela aparece quando há mais de uma série e em `pie` e `donut`. Use `legend=false` para escondê-la. |
| `stacked` | Empilha as séries em vez de colocá-las lado a lado. Vale para gráficos de barras. |
| `colors="marca coral"` | Cores das séries, na ordem das colunas, separadas por espaço. Aceita cores do tema e `#hex`. |
| `w`, `h` | Largura e altura do gráfico, como `h=480` ou `w=60%`. |

Os atributos de sim ou não (`values`, `legend`, `stacked`) podem ser escritos sozinhos para ligar ou com `=false` para desligar.

```ribalta preview
@color marca #1e83d6

# Custos por área
::: chart {stacked values colors="marca coral" h=640}
| Área | Pessoas | Ferramentas |
| --- | ---: | ---: |
| Vendas | 40 | 12 |
| Suporte | 25 | 8 |
| Produto | 55 | 20 |
:::
```

:::note
Sem `colors`, as séries usam a paleta do tema, na ordem. Em `pie` e `donut`, as fatias sempre usam a paleta do tema.
:::

## Gráficos a partir de dados

Com uma tabela declarada em `@data`, o gráfico pode ter uma linha só. `label` escolhe o campo dos rótulos e `series` os campos com valores:

```ribalta preview
@data vendas
| trimestre | receita | custo |
| ---       | ---     | ---   |
| T1        | 120     | 80    |
| T2        | 150     | 95    |
| T3        | 190     | 110   |
@end

:: chart {data=$vendas type=area series=receita}
```

Os mesmos dados podem alimentar outros gráficos, tabelas (`:: table {data=$vendas}`) e repetições. Veja [Variáveis, dados e repetição](/docs/guides/data/).

## Números

- Use ponto ou vírgula como separador decimal: `3.5` e `3,5` valem o mesmo.
- Um `%` no fim é aceito e ignorado: `38%` vale `38`.
- Espaços dentro do número são ignorados: `1 200` vale `1200`.
- Não use ponto como separador de milhar: `1.200` é lido como um vírgula dois.
- Uma célula que falta numa linha vale zero.

## Erros comuns

| Código | Quando acontece |
| --- | --- |
| R111 | O bloco `chart` não tem uma tabela. |
| R112 | Uma célula de valor não é um número, como `abc` ou `R$ 10`. O gráfico usa zero no lugar. |
| R113 | A tabela tem só a coluna de rótulos, sem nenhuma coluna de valores. |
| R301 | Tipo de gráfico inválido, cor desconhecida em `colors` ou valor inválido em `values`, `legend` ou `stacked`. |
