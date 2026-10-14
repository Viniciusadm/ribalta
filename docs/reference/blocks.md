---
title: Blocos
description: Todos os blocos embutidos da linguagem, com forma, conteúdo, atributos próprios e exemplos.
---

Blocos organizam o conteúdo da página. Há duas formas:

| Forma | Sintaxe | Uso |
| --- | --- | --- |
| Contêiner | `::: nome {atributos}` … `:::` | Blocos com conteúdo. Um `:::` sozinho na linha fecha o último bloco aberto. |
| Uma linha | `:: nome valor {atributos}` | Blocos sem conteúdo, como ícones e espaços. |

Todos os blocos aceitam os [atributos de estilo](/docs/reference/attributes/#atributos-de-estilo). Blocos contêiner podem ser aninhados à vontade, e a indentação é opcional.

## Resumo

| Bloco | Forma | Filhos | Atributos próprios |
| --- | --- | --- | --- |
| [`columns`](#columns) | Contêiner | Cada filho é uma coluna | `widths` |
| [`stack`](#stack) | Contêiner | Empilhados na vertical | |
| [`row`](#row) | Contêiner | Lado a lado, na horizontal | `wrap` |
| [`grid`](#grid) | Contêiner | Cada filho é uma célula | `cols` |
| [`box`](#box) | Contêiner | Empilhados dentro da caixa | |
| [`steps`](#steps) | Contêiner | Revelados um por vez | |
| [`notes`](#notes) | Contêiner | Texto puro das notas | |
| [`chart`](#chart) | Contêiner ou uma linha | Uma tabela com os dados, ou `data=$lista` | `type`, `legend`, `stacked`, `values`, `colors`, `data`, `label`, `series` |
| [`table`](#table) | Uma linha | Nenhum; os dados vêm de `data=$lista` | `data`, `columns` |
| [`if` e `else`](#if-e-else) | Contêiner | O conteúdo mostrado conforme a condição | |
| [`for`](#for) | Contêiner | O conteúdo repetido para cada item | |
| [`slot`](#slot) | Contêiner | Conteúdo de um espaço nomeado | |
| [`icon`](#icon) | Uma linha | Nenhum; o valor é o nome do ícone | |
| [`spacer`](#spacer) | Uma linha | Nenhum; o valor é o tamanho | |
| [`divider`](#divider) | Uma linha | Nenhum | |
| [`input`](#input) | Uma linha | Nenhum | `bind`, `placeholder`, `on-submit` |
| [`sound`](#sound) | Uma linha | Nenhum; o valor é o endereço do áudio | `name`, `autoplay`, `loop`, `volume` |

`icon`, `spacer` e `divider` não aceitam conteúdo ([R109](/docs/reference/errors/)). Nos demais blocos embutidos, texto depois do nome é ignorado ([R110](/docs/reference/errors/)); use atributos entre chaves.

## columns

Coloca os filhos lado a lado. **Cada filho direto é uma coluna**: um parágrafo, uma imagem ou, para agrupar vários elementos numa coluna, um `::: stack`.

| Atributo | Valores | Padrão |
| --- | --- | --- |
| `widths` | Proporções separadas por espaço, uma por coluna: `widths="2 1"` | Colunas iguais |
| `gap` | Espaço entre as colunas | `md` (32 px) |
| `valign` | Alinhamento vertical das colunas | `center` |

```ribalta preview
::: columns {widths="2 1" gap=xl}
::: stack {gap=sm}
## Plano de expansão
Duas regiões novas até dezembro,
com a mesma equipe de suporte.
:::
::: box {.card align=center}
:: icon map-pin {color=accent}
2 regiões
:::
:::
```

## stack

Empilha os filhos na vertical. É o bloco usado para agrupar vários elementos dentro de uma coluna ou de uma célula.

| Atributo | Uso | Padrão |
| --- | --- | --- |
| `gap` | Espaço entre os filhos | `md` (32 px) |
| `align` | Alinhamento horizontal dos filhos | Ocupam toda a largura |
| `valign` | Alinhamento vertical dos filhos | `top` |

```ribalta preview
::: stack {gap=xs align=center}
Capítulo 1 {.kicker}
# O começo de tudo
Uma história em três atos {.muted}
:::
```

## row

Coloca os filhos em linha, na horizontal, com o tamanho do próprio conteúdo. É útil para ícones, selos e listas curtas de itens.

| Atributo | Valores | Padrão |
| --- | --- | --- |
| `wrap` | Booleano: quebra os itens em mais de uma linha quando não cabem | Desligado |
| `gap` | Espaço entre os itens | 16 px |
| `align` | Distribuição horizontal: `left`, `center`, `right` | `left` |
| `valign` | Alinhamento vertical dos itens | `center` |

```ribalta preview
::: row {gap=md align=center}
:: icon star {size=lg color=yellow}
:: icon star {size=lg color=yellow}
:: icon star {size=lg color=yellow}
4,8 de 5 em 1.200 avaliações {size=xl}
:::
```

## grid

Grade com número fixo de colunas. Cada filho ocupa uma célula, preenchendo as linhas da esquerda para a direita.

| Atributo | Valores | Padrão |
| --- | --- | --- |
| `cols` | Número de colunas, de 1 a 12 | Número de filhos, limitado a 3 |
| `gap` | Espaço entre as células | `md` (32 px) |
| `valign` | Alinhamento vertical das células | Esticadas para a mesma altura |

```ribalta preview
::: grid {cols=3 gap=lg}
::: box {.card}
### Rápido
Prévia a cada tecla.
:::
::: box {.card}
### Consistente
Temas e estilos reutilizáveis.
:::
::: box {.card}
### Versionado
Histórico de cada alteração.
:::
:::
```

## box

Caixa que empilha os filhos na vertical e recebe fundo, borda, cantos e espaçamento. Combine com o estilo `.card` (`bg=surface radius=lg pad=lg shadow=md`) ou monte o visual com atributos.

| Atributo | Uso | Padrão |
| --- | --- | --- |
| `gap` | Espaço entre os filhos | 20 px |
| `bg`, `border`, `radius`, `pad`, `shadow` | Aparência da caixa | Sem fundo, sem borda |

```ribalta preview
::: columns {gap=lg}
::: box {.card}
### Com `.card`
Fundo, cantos e sombra prontos.
:::
::: box {border="3 accent" radius=md pad=lg}
### Com atributos
Só borda e espaçamento.
:::
:::
```

## steps

Revela os filhos um por vez enquanto o documento é mostrado: cada filho é uma etapa. Se um filho é uma lista, **cada item** da lista vira uma etapa. Numa visualização estática, como uma imagem ou um PDF, tudo aparece de uma vez.

Aceita os mesmos atributos de `stack`. Para controlar a ordem manualmente, use `{step=N}` em qualquer bloco ou trecho. Veja [Revelação](/docs/guides/reveal/).

```ribalta preview
## Três motivos
::: steps {gap=sm}
- Menos tempo formatando
- Mais tempo no conteúdo
- Apresentações consistentes
:::
```

## notes

Notas para quem mostra o documento, como as notas do apresentador numa apresentação. O conteúdo nunca é desenhado na página: o programa que mostra o documento decide onde exibi-lo.

- O texto é guardado como está, sem interpretar formatação nem blocos.
- Vários blocos `notes` na mesma página são juntados, separados por uma linha em branco.
- Atributos na linha `::: notes` são ignorados.

```ribalta
# Resultados
Crescemos 38% no trimestre.

::: notes
Fazer uma pausa depois do número.
Perguntar se alguém tem dúvidas.
:::
```

## chart

Desenha um gráfico a partir de uma tabela Markdown dentro do bloco. A primeira coluna tem os rótulos; cada coluna seguinte é uma série, e o cabeçalho dá o nome da série. As células aceitam números como `42`, `3.5`, `3,5` e `12%`.

| Atributo | Valores | Padrão |
| --- | --- | --- |
| `type` | `bar`, `line`, `area`, `pie`, `donut` | `bar` |
| `legend` | Booleano | Ligado com mais de uma série ou em `pie`/`donut` |
| `stacked` | Booleano: empilha as séries | Desligado |
| `values` | Booleano: mostra os valores | Desligado |
| `colors` | Cores separadas por espaço, uma por série: `colors="accent blue"` | Paleta do tema |

```ribalta preview
::: chart {type=bar values}
| Mês | 2025 | 2026 |
| --- | ---: | ---: |
| Jan | 32 | 41 |
| Fev | 28 | 45 |
| Mar | 35 | 52 |
:::
```

Com `data=$lista`, os dados vêm de um `@data` e o bloco pode ter uma linha só: `:: chart {data=$vendas label=mes series="receita custo"}`.

Mais exemplos em [Gráficos](/docs/guides/charts/).

## table

Monta uma tabela a partir de uma lista de registros. `columns` escolhe os campos e a ordem; colunas só com números ficam alinhadas à direita.

```ribalta preview
@data equipe
| nome  | cargo      | anos |
| ---   | ---        | ---  |
| Ana   | CTO        | 7    |
| Bruno | Design     | 3    |
@end

:: table {data=$equipe columns="nome anos"}
```

Para uma tabela escrita à mão, use a sintaxe Markdown `| A | B |`.

## if e else

`::: if condição` mostra o conteúdo quando a condição é verdadeira. Logo depois podem vir `::: else if condição` e `::: else`. O conteúdo escolhido entra no bloco pai, sem caixa extra.

```ribalta preview
@let versao = curta

::: if $versao == curta
Resumo em uma frase.
:::
::: else
Versão completa, com todos os detalhes.
:::
```

O atributo `if=condição` faz o mesmo para um único elemento. Com `@state`, a escolha acontece enquanto o documento é mostrado; veja [Interatividade](/docs/guides/interactive/).

## for

`::: for item in $lista` repete o conteúdo para cada item, e os blocos gerados entram no bloco pai. `::: for item, i in $lista` também dá a posição (a partir de 1) e `::: for i in 1..5` repete para um intervalo.

```ribalta preview
::: row {gap=md}
::: for n in 1..5
:: icon star {color="$($n <= 4 ? accent : muted)"}
:::
:::
```

Listas Markdown geradas por um `for` são juntadas numa lista só. Veja [Variáveis, dados e repetição](/docs/guides/data/).

## slot

Preenche um espaço nomeado de um componente. Só pode ser usado diretamente dentro do uso de um componente contêiner ([R416](/docs/reference/errors/) fora dele). O nome do espaço vem depois de `slot`.

```ribalta preview
@component comparacao {antes depois}
::: columns {gap=xl}
::: box {.card}
### $antes
$content
:::
::: box {.card}
### $depois
$lado-b
:::
:::
@end

::: comparacao {antes=Planilhas depois="Painel vivo"}
Exportar, colar e formatar à mão.
::: slot lado-b
Os dados chegam sozinhos.
:::
:::
```

Veja [Componentes](/docs/guides/components/) para os detalhes de `$content` e dos espaços nomeados.

## icon

Desenha um ícone da biblioteca embutida. O nome vem depois de `icon`; a lista completa está em [Ícones](/docs/reference/icons/).

| Atributo | Uso | Padrão |
| --- | --- | --- |
| `size` | Tamanho do ícone: escala `xs`…`7xl` ou pixels | 96 px |
| `color` | Cor do traço | Cor do texto |

```ribalta preview
::: row {gap=xl align=center}
:: icon rocket {size=3xl color=accent}
:: icon lightbulb {size=3xl color=yellow}
:: icon chart-line {size=3xl color=blue}
:::
```

## spacer

Espaço vazio entre dois blocos. O valor é um tamanho da [escala de espaçamento](/docs/reference/attributes/#espaçamento) (`none` a `3xl`) ou pixels de 0 a 600. Sem valor, vale `md` (32 px).

```ribalta preview
# Antes do espaço
:: spacer 3xl
Depois de 160 px {.muted}
```

## divider

Linha divisória horizontal. Aceita atributos de estilo, como `w` para limitar a largura e `opacity`.

```ribalta preview
## Seção
:: divider {w=40%}
Texto depois da linha {.muted}
```

## input

Campo de texto ligado a um `@state`. O que a pessoa digita vai direto para a variável, então o texto em volta muda na hora. Ligado a um `@state` de número, o campo aceita só números.

| Atributo | Uso | Padrão |
| --- | --- | --- |
| `bind` | A variável que guarda o valor, como `bind=$nome` (obrigatório) | |
| `placeholder` | Texto de exemplo mostrado com o campo vazio | Nenhum |
| `on-submit` | Ações ao apertar Enter, inclusive `go` | Nenhuma |

```ribalta preview
@state nome = ""

## Como você se chama?
:: input {bind=$nome placeholder="Seu nome"}

Olá, $nome! {if="$nome != \"\""}
```

Digitar no campo não avança a página. Numa visualização estática, como uma imagem ou um PDF, o campo aparece com o valor inicial. Veja [Entrada de texto](/docs/guides/interactive/#entrada-de-texto).

## sound

Um som da página. Ele não é desenhado: toca com `autoplay` ao entrar, ou quando uma ação `play` pede. Todos os sons param ao sair da página. Visualizações estáticas, como imagens e PDF, não tocam som.

| Atributo | Uso | Padrão |
| --- | --- | --- |
| `name` | Nome usado por `play` e `stop`, como `name=porta` | Nenhum |
| `autoplay` | Toca assim que a página abre | Desligado |
| `loop` | Repete sem parar | Desligado |
| `volume` | De 0 a 1, ou porcentagem | `1` |

```ribalta
:: sound https://exemplo.com/chuva.mp3 {name=chuva loop volume=40%}
:: sound https://exemplo.com/porta.mp3 {name=porta}
# A porta
[Abrir]{click="play porta"} · [Silêncio]{click="stop chuva"}
```

O endereço começa com `https://`. Veja [Som](/docs/guides/interactive/#som).
