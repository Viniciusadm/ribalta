---
title: Atributos
description: Todos os atributos aceitos pelo compilador, com valores, escalas e unidades.
---

Atributos ficam entre chaves, como `{size=5xl color=accent}`. Esta página lista todos os atributos que o compilador aceita e os valores válidos de cada um. Para a sintaxe das chaves (`.estilo`, `#id`, aspas, posição na linha), veja [Sintaxe](/reference/syntax/#atributos).

Um atributo desconhecido é erro ([R209](/reference/errors/)) e um valor inválido também ([R301](/reference/errors/)); as mensagens sugerem o nome ou valor mais próximo.

## Atributos de estilo

Valem em qualquer bloco, no separador de página (`--- {…}`), em trechos de texto (`[texto]{…}`) e em imagens.

| Atributo | Valores aceitos | Exemplo | Observações |
| --- | --- | --- | --- |
| `color` | [Cor](#cores) | `color=accent` | Cor do texto. Também vale para ícones. |
| `bg` | [Cor](#cores) ou gradiente `cor..cor` | `bg=ink..violet` | Fundo. O gradiente é linear, em diagonal (135°). |
| `image` | `https://…`, `http://…`, `data:image/…` ou `asset:<id>` | `image=asset:<id>` | Imagem de fundo, cobrindo a área e centralizada. |
| `font` | Token de fonte: `body`, `display`, `mono` ou um `@font` | `font=display` | Família tipográfica. |
| `size` | [Escala de tamanhos](#tamanhos) ou pixels de 8 a 400 | `size=5xl`, `size=120` | Tamanho do texto; em ícones, o tamanho do ícone. |
| `weight` | [Peso por nome](#pesos) ou de 100 a 900, de 100 em 100 | `weight=bold`, `weight=600` | Peso da fonte. |
| `italic` | Booleano | `italic`, `italic=false` | Itálico. |
| `uppercase` | Booleano | `uppercase` | Caixa alta. |
| `align` | `left`, `center`, `right` | `align=center` | Alinhamento **horizontal**, em qualquer layout. |
| `valign` | `top`, `center` (ou `middle`), `bottom` | `valign=bottom` | Alinhamento **vertical**, em qualquer layout. |
| `pad` | Um ou dois valores de [espaçamento](#espaçamento) | `pad=lg`, `pad="sm xl"` | Espaço interno. Com dois valores: vertical e horizontal. |
| `gap` | [Espaçamento](#espaçamento) | `gap=xl` | Espaço entre os filhos de um bloco. |
| `radius` | [Arredondamento](#arredondamento) ou pixels de 0 a 9999 | `radius=lg` | Cantos arredondados. |
| `border` | Espessura (0 a 40 px) e/ou [cor](#cores), em qualquer ordem | `border`, `border="2 coral"` | Sem valor: 3 px na cor `line` do tema. |
| `shadow` | `sm`, `md`, `lg`, `none` | `shadow`, `shadow=lg` | Sem valor, vale `md`. |
| `w` | [Medida](#medidas) ou `auto` | `w=50%`, `w=480` | Largura. |
| `h` | [Medida](#medidas) ou `auto` | `h=full` | Altura. |
| `x` | [Medida](#medidas) | `x=120` | Posição horizontal absoluta (distância da borda esquerda da página). |
| `y` | [Medida](#medidas) | `y=80` | Posição vertical absoluta (distância do topo da página). Com só um de `x`/`y`, o outro vale 0. |
| `opacity` | Número de 0 a 1 ou porcentagem de 0% a 100% | `opacity=0.6`, `opacity=60%` | Transparência do bloco inteiro. |
| `leading` | `tight`, `normal`, `loose` ou número de 0.6 a 3 | `leading=tight` | Altura da linha. Veja [entrelinha](#entrelinha-e-espaçamento-entre-letras). |
| `tracking` | `tight`, `normal`, `wide` ou número de -0.2 a 1 (em `em`) | `tracking=0.1` | Espaço entre letras. |
| `step` | Número inteiro de 1 a 99 | `step=2` | Etapa em que o bloco ou trecho aparece. Veja [Revelação](/guides/reveal/). |
| `enter` | `fade`, `up`, `down`, `left`, `right`, `zoom`, `none` | `enter=zoom` | Como o elemento aparece numa etapa ou quando um `if` fica verdadeiro. Padrão nas etapas: `up`. |
| `if` | Condição | `if=$detalhes`, `if="$n > 2"` | Mostra o elemento só quando a condição é verdadeira. Veja [Variáveis](/guides/data/#condições-com--if). |
| `click` | Ações separadas por `;` | `click="go #fim"` | O que acontece ao clicar. Veja [Interatividade](/guides/interactive/#cliques). |
| `key` | Uma letra, um número, `up`, `down`, `left`, `right`, `Enter` ou `Space` | `key=up` | Tecla que aciona o `click` do elemento. Veja [Teclado](/guides/interactive/#teclado). |

Além disso, `.estilo=condição` aplica um estilo só quando a condição é verdadeira: `{.destaque="$plano == pro"}`. Com `.estilo=hover`, o estilo vale enquanto o mouse está sobre o elemento.

Valores de atributos aceitam `$variavel` e `$( expressão )`, desde que não dependam de `@state` ([R604](/reference/errors/)).

Atributos booleanos aceitam `true`, `yes` ou `sim` para ligar e `false`, `no` ou `não` para desligar. Escrever só o nome (`{italic}`) liga o atributo.

```ribalta preview
--- {bg=paper..surface}
::: columns {gap=xl}
::: box {bg=accent color=white radius=xl pad="md lg" shadow=lg}
## Cartão quente
Fundo `accent`, cantos `xl`
:::
::: box {border="4 blue" radius=md pad=lg}
## Borda azul {color=blue}
`border="4 blue"` {italic}
:::
::: box {bg=ink..violet color=white radius=full pad="lg xl" align=center}
## Gradiente
`bg=ink..violet` {opacity=0.8}
:::
:::
```

## Atributos de cada bloco

Além dos atributos de estilo, alguns blocos aceitam atributos próprios. Usá-los em outro lugar é erro ([R209](/reference/errors/)). Em um `@style`, todos são aceitos e só valem quando o estilo é aplicado ao bloco certo.

| Atributo | Onde | Valores | Padrão | Exemplo |
| --- | --- | --- | --- | --- |
| `fit` | Imagem em bloco | `contain` (inteira) ou `cover` (preenche e corta) | `contain` | `![Foto](https://…){fit=cover}` |
| `widths` | `columns` | Números positivos separados por espaço, um por coluna | Colunas iguais | `::: columns {widths="2 1"}` |
| `cols` | `grid` | Número inteiro de 1 a 12 | Número de filhos, de 1 a 3 | `::: grid {cols=4}` |
| `wrap` | `row` | Booleano: quebra os itens em mais de uma linha | Desligado | `::: row {wrap}` |
| `type` | `chart` | `bar`, `line`, `area`, `pie`, `donut` | `bar` | `::: chart {type=line}` |
| `legend` | `chart` | Booleano | Ligado com mais de uma série ou em `pie`/`donut` | `{legend=false}` |
| `stacked` | `chart` | Booleano: empilha as séries | Desligado | `{stacked}` |
| `values` | `chart` | Booleano: mostra os valores no gráfico | Desligado | `{values}` |
| `colors` | `chart` | Lista de [cores](#cores) separadas por espaço, uma por série | Paleta do tema | `{colors="accent blue"}` |
| `data` | `chart`, `table` | Uma lista de registros (`@data`) | | `:: chart {data=$vendas}` |
| `label` | `chart` com `data` | O campo com os rótulos | O primeiro campo de texto | `{label=mes}` |
| `series` | `chart` com `data` | Campos com os valores, separados por espaço | Todos os campos numéricos | `{series="receita custo"}` |
| `columns` | `table` | Campos exibidos, em ordem | Todos | `{columns="nome cargo"}` |
| `width` | Separador `---` | Largura desta página em pixels, de 16 a 16384 | `@page` ou o padrão do programa | `--- {width=1080}` |
| `height` | Separador `---` | Altura desta página em pixels, de 16 a 16384, ou `auto` para crescer com o conteúdo | `@page` ou o padrão do programa | `--- {width=1080 height=1920}` |
| `cover` | Separador `---` | Booleano: a página é capa e fica sem cabeçalho, rodapé e camada livre | Ligado só na primeira página | `--- {cover}` |
| `header` | Separador `---` | `on` ou `off`: mostra ou esconde o cabeçalho nesta página | Ligado, menos na capa | `--- {header=off}` |
| `footer` | Separador `---` | `on` ou `off`: mostra ou esconde o rodapé nesta página | Ligado, menos na capa | `--- {footer=on}` |
| `overlay` | Separador `---` | `on` ou `off`: mostra ou esconde a camada livre nesta página | Ligado, menos na capa | `--- {overlay=off}` |
| `hidden` | Separador `---` | Booleano: a página sai do fluxo normal e só é alcançada por link ou `go` | Desligado | `--- {#extra hidden}` |
| `next` | Separador `---` | Id da página seguinte ao avançar | A próxima página | `--- {next=#fim}` |
| `steps` | Separador `---` | Número mínimo de etapas da página, de 1 a 99 | Calculado | `--- {steps=3}` |
| `transition` | Separador `---` | `fade`, `slide`, `zoom`, `none` | `@transition` ou nenhuma | `--- {transition=slide}` |
| `timeout` | Separador `---` | Segundos (1 a 3600) até rodar `on-timeout` | Nenhum | `--- {timeout=30}` |
| `on-timeout` | Separador `---` | Ações ao completar `timeout`, inclusive `go` | Nenhuma | `--- {timeout=30 on-timeout="go #fim"}` |
| `nav` | Separador `---` | `on` ou `off`: com `off`, setas, espaço, clique e deslizar não saem da página | `on` (`off` com `@mode game`) | `--- {nav=off}` |
| `on-enter` | Separador `---` | Ações de estado separadas por `;` (`$x = valor`, `toggle $x`) | Nenhuma | `--- {on-enter="$visto = true"}` |

Em `widths`, as proporções são relativas: `"2 1"` dá duas partes para a primeira coluna e uma para a segunda. Se a quantidade de valores não bate com a de colunas, o compilador avisa ([R302](/reference/errors/)).

A paleta padrão dos gráficos usa, na ordem, `accent`, `blue`, `green`, `yellow`, `violet` e `coral`.

## Escalas e unidades

### Tamanhos

Usados por `size`. Também é possível informar pixels de 8 a 400.

| Nome | Pixels |
| --- | --- |
| `xs` | 20 |
| `sm` | 26 |
| `md` | 32 |
| `lg` | 40 |
| `xl` | 52 |
| `2xl` | 64 |
| `3xl` | 88 |
| `4xl` | 116 |
| `5xl` | 150 |
| `6xl` | 200 |
| `7xl` | 260 |

```ribalta preview
::: row {gap=lg valign=bottom}
md {size=md}
xl {size=xl}
3xl {size=3xl}
5xl {size=5xl}
7xl {size=7xl}
:::
```

### Espaçamento

Usados por `pad`, `gap` e `:: spacer`. Também é possível informar pixels de 0 a 600.

| Nome | Pixels |
| --- | --- |
| `none` | 0 |
| `xs` | 8 |
| `sm` | 16 |
| `md` | 32 |
| `lg` | 48 |
| `xl` | 72 |
| `2xl` | 112 |
| `3xl` | 160 |

### Arredondamento

Usados por `radius`. Também é possível informar pixels de 0 a 9999.

| Nome | Pixels |
| --- | --- |
| `none` | 0 |
| `sm` | 8 |
| `md` | 16 |
| `lg` | 28 |
| `xl` | 44 |
| `full` | 9999 (formato de pílula ou círculo) |

### Pesos

Usados por `weight`. Também é possível informar números de 100 a 900, de 100 em 100.

| Nome | Peso |
| --- | --- |
| `thin` | 100 |
| `light` | 300 |
| `regular` | 400 |
| `medium` | 500 |
| `semibold` | 600 |
| `bold` | 700 |
| `extrabold` | 800 |
| `black` | 900 |

O peso só aparece se a fonte tiver essa variação. Fontes declaradas com `@font` carregam os pesos listados na definição.

### Entrelinha e espaçamento entre letras

| Atributo | Palavra | Valor | Faixa numérica |
| --- | --- | --- | --- |
| `leading` | `tight` | 1.05 | 0.6 a 3 (multiplica o tamanho da fonte) |
| `leading` | `normal` | 1.35 | |
| `leading` | `loose` | 1.7 | |
| `tracking` | `tight` | -0.02 em | -0.2 a 1 (em `em`) |
| `tracking` | `normal` | 0 | |
| `tracking` | `wide` | 0.08 em | |

### Medidas

Usadas por `w`, `h`, `x` e `y`.

| Forma | Exemplo | Significado |
| --- | --- | --- |
| Pixels | `480` ou `480px` | De -4000 a 4000. O sufixo `px` é opcional. |
| Porcentagem | `50%` | De 0% a 100% do espaço disponível. |
| `full` | `w=full` | O mesmo que `100%`. |
| `auto` | `h=auto` | Tamanho automático. Só em `w` e `h`. |

As medidas são pixels da página, qualquer que seja a tela em que ela aparece. O tamanho da página vem de `@page` (por exemplo, `@page 1920x1080`) ou de `width` e `height` no separador; sem eles, vale o padrão do programa que compila o documento. Um programa de apresentações costuma usar 1920×1080; sem nenhum padrão, a página tem 1920 de largura e altura automática. Com altura automática (`height=auto`), a página cresce com o conteúdo. Veja [Tamanhos e documentos](/guides/pages/).

### Cores

Usadas por `color`, `bg`, `border` e `colors`.

| Forma | Exemplo | Significado |
| --- | --- | --- |
| Token do tema | `accent`, `muted`, `coral` | Cor definida pelo tema ou por `@color`. |
| Hexadecimal | `#f60`, `#ff6600`, `#ff660080` | `#rgb`, `#rgba`, `#rrggbb` ou `#rrggbbaa`. |
| Opacidade | `accent/40`, `#000000/20` | Cor com opacidade de 0 a 100 (%), depois da barra. |
| Gradiente | `ink..violet`, `accent/80..blue` | Duas cores separadas por `..`. **Só em `bg`.** |

Tokens disponíveis em todos os temas: `bg`, `text`, `muted`, `accent`, `surface`, `line`, `coral`, `blue`, `green`, `yellow`, `violet`, `white`, `black`, `ink`, `paper` e `transparent`. Os valores de cada tema estão em [Temas embutidos](/reference/themes/).

Todo token de cor também funciona como estilo: `{.coral}` é o mesmo que `{color=coral}`.

```ribalta preview
::: grid {cols=4 gap=md}
::: box {bg=accent pad=md radius=md}
`accent`
:::
::: box {bg=accent/40 pad=md radius=md}
`accent/40`
:::
::: box {bg=#2f6fdb color=white pad=md radius=md}
`#2f6fdb`
:::
::: box {bg=coral..violet color=white pad=md radius=md}
`coral..violet`
:::
:::
```
