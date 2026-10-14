---
title: Layouts
description: Configure a página, organize o conteúdo em colunas, pilhas, linhas e grades e posicione elementos com precisão.
---

Por padrão, o conteúdo de uma página é empilhado de cima para baixo, alinhado à esquerda e centralizado na vertical. Para mudar isso, você configura a página na linha `---` e usa blocos de layout.

## Atributos da página

Atributos entre chaves na linha `---` valem para a página inteira:

```ribalta preview
--- {align=center valign=bottom bg=blue color=white pad="lg 2xl"}
# Encerramento
Obrigado pela atenção.
```

Os mais usados:

| Atributo | O que faz | Exemplo |
| --- | --- | --- |
| `align` | Alinhamento horizontal do conteúdo: `left`, `center` ou `right` | `align=center` |
| `valign` | Alinhamento vertical do conteúdo: `top`, `center` ou `bottom` | `valign=top` |
| `bg` | Cor ou gradiente de fundo | `bg=ink..violet` |
| `color` | Cor do texto | `color=white` |
| `image` | Imagem de fundo | `image=https://…` |
| `pad` | Margem interna: um valor, ou dois (vertical e horizontal) | `pad="lg 2xl"` |
| `gap` | Espaço entre os blocos da página | `gap=lg` |
| `width`, `height` | Tamanho desta página, em pixels (`height=auto` cresce com o conteúdo) | `width=1080 height=1920` |
| `#id` | Identificador da página | `#capa` |
| `.estilo` | Aplica um estilo pronto ou seu | `.dark` |

A página aceita qualquer atributo de estilo. Veja a lista completa em [Atributos](/reference/attributes/). Para mudar o tamanho das páginas, veja [Tamanhos e documentos](/guides/pages/).

:::tip
Para configurar a primeira página, comece o texto com uma linha `--- {…}`. Ela não cria uma página vazia antes dela.
:::

## Blocos de layout

Um bloco de layout começa com `:::` seguido do nome e termina com uma linha contendo apenas `:::`. Os atributos do bloco ficam entre chaves na linha de abertura:

```ribalta
::: columns {gap=xl}
Conteúdo da primeira coluna

Conteúdo da segunda coluna
:::
```

Cada bloco direto dentro de um layout é um item: um título, um parágrafo, uma lista, uma imagem ou outro bloco de layout. Um `:::` sozinho sempre fecha o último bloco aberto. A indentação é opcional e serve só para facilitar a leitura.

## Colunas

`::: columns` coloca cada item lado a lado. Use `widths` para as proporções e `gap` para o espaço entre as colunas:

```ribalta preview
# Por que mudar agora
::: columns {widths="2 1" gap=xl valign=top}
::: stack {gap=sm}
## O cenário
O mercado dobrou em dois anos.
Os clientes pedem respostas mais rápidas.
:::
::: box {.card}
## Meta
Reduzir o prazo pela metade.
:::
:::
```

| Atributo | O que faz |
| --- | --- |
| `widths` | Proporção de cada coluna, separadas por espaço. `widths="2 1"` faz a primeira coluna ter o dobro da largura da segunda. Sem ele, as colunas dividem o espaço igualmente. |
| `gap` | Espaço entre as colunas. |
| `valign` | Alinhamento vertical das colunas: `top`, `center` (padrão) ou `bottom`. |

:::note
O número de valores em `widths` deve ser igual ao número de colunas. Se não for, o compilador avisa.
:::

## Pilha

`::: stack` empilha os itens de cima para baixo. Ela é útil para agrupar vários blocos em uma única coluna ou para controlar o espaço entre eles com `gap`:

```ribalta preview
::: stack {gap=xs}
Capítulo 1 {.kicker}
# Onde estamos
Um panorama do mercado em 2026. {.lead}
:::
```

## Linha

`::: row` coloca os itens lado a lado, cada um com a largura do próprio conteúdo. Com `wrap`, os itens que não cabem passam para a linha seguinte:

```ribalta preview
# Tecnologias
::: row {gap=lg wrap}
:: icon database {size=3xl color=accent}
:: icon cloud {size=3xl color=blue}
:: icon shield {size=3xl color=green}
:: icon cpu {size=3xl color=violet}
:::
```

| Atributo | O que faz |
| --- | --- |
| `gap` | Espaço entre os itens. |
| `wrap` | Quebra a linha quando os itens não cabem. |
| `align` | Posição horizontal dos itens: `left`, `center` ou `right`. |
| `valign` | Alinhamento vertical dos itens: `top`, `center` (padrão) ou `bottom`. |

## Grade

`::: grid` distribui os itens em colunas de mesma largura e cria novas linhas quando necessário. Defina o número de colunas com `cols`, de 1 a 12. Sem `cols`, a grade usa uma coluna por item, até no máximo três:

```ribalta preview
# Indicadores
::: grid {cols=2 gap=md}
::: box {.card}
## 94
clientes ativos {.muted}
:::
::: box {.card}
## 38%
crescimento anual {.muted}
:::
::: box {.card}
## 4,8
nota de satisfação {.muted}
:::
::: box {.card}
## 12
países atendidos {.muted}
:::
:::
```

## Caixa

`::: box` agrupa itens empilhados, como a pilha, e é a forma mais simples de dar fundo, borda, cantos arredondados e sombra a um conjunto de blocos. O estilo pronto `.card` aplica tudo isso de uma vez:

```ribalta preview
::: box {.card align=center gap=xs}
:: icon rocket {size=3xl color=accent}
## Lançamento
Disponível para todos os clientes em março.
:::
```

## Aninhando blocos

Blocos de layout podem ficar uns dentro dos outros. Cada `:::` sozinho fecha o bloco aberto mais recente, então a estrutura é lida de dentro para fora:

```ribalta preview
::: columns {gap=xl}
::: stack {gap=sm}
## Antes
- Relatórios manuais
- Dados desencontrados
:::
::: box {.card gap=sm}
## Depois
::: row {gap=sm}
:: icon check {size=xl color=green}
Relatórios automáticos
:::
::: row {gap=sm}
:: icon check {size=xl color=green}
Uma única fonte de dados
:::
:::
:::
```

## Alinhamento

Na Ribalta, `align` é **sempre horizontal** e `valign` é **sempre vertical**, em qualquer bloco e em qualquer direção de layout. Você não precisa pensar em eixos:

```ribalta preview
::: columns {gap=lg}
::: box {.card h=600 align=center valign=center}
## Centro
:::
::: box {.card h=600 align=right valign=bottom}
## Canto
:::
:::
```

| Atributo | Valores |
| --- | --- |
| `align` | `left`, `center`, `right` |
| `valign` | `top`, `center`, `bottom` |

## Tamanho e posição absoluta

Os atributos `w` e `h` definem largura e altura. Eles aceitam pixels (`480`), porcentagem (`50%`), `full` ou `auto`.

Os atributos `x` e `y` tiram um bloco do fluxo e o posicionam em um ponto exato da página, medido em pixels a partir do canto superior esquerdo dela. Numa página de 1920 × 1080, como a deste exemplo, o canto inferior direito fica em `x=1920 y=1080`:

```ribalta preview
# Lançamento em março
O cronograma está confirmado para todas as equipes.

Confidencial {x=1560 y=64 .kicker}

::: box {x=1360 y=760 w=440 bg=accent color=white pad=md radius=lg}
Prazo final: 31/03
:::
```

:::caution
Elementos posicionados com `x` e `y` não empurram os outros blocos e podem ficar por cima deles. Use com moderação, para selos, marcas e detalhes.
:::

## Espaçador e divisor

Dois blocos de uma linha só ajudam a ajustar o ritmo da página:

- `:: spacer` cria um espaço vazio. Informe o tamanho depois do nome, usando a escala de espaçamento (`xs` a `3xl`) ou pixels. Sem valor, o espaço é `md`.
- `:: divider` desenha uma linha horizontal. Mude a cor com `bg` e a largura com `w`.

```ribalta preview
# Agenda
:: divider {w=160 bg=accent}
:: spacer lg
1. Contexto
2. Proposta
3. Próximos passos
```

## Próximos passos

- [Estilos](/guides/styling/): cores, tamanhos, bordas e estilos reutilizáveis.
- [Componentes](/guides/components/): transforme layouts que se repetem em blocos reutilizáveis.
- [Referência de blocos](/reference/blocks/): todos os blocos e seus atributos.
