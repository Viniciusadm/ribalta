---
title: Estilos
description: Mude cores, tamanhos, fontes, bordas e sombras com atributos entre chaves e crie estilos reutilizáveis.
---

A aparência de qualquer bloco ou trecho de texto é ajustada com atributos entre chaves. Os valores vêm de escalas e nomes do tema, o que mantém as páginas consistentes sem esforço.

## Atributos entre chaves

Os atributos têm a forma `{.estilo chave=valor chave="valor com espaços" opção}`. Há dois lugares para colocá-los:

| Onde | Como | Vale para |
| --- | --- | --- |
| No fim de uma linha, **com espaço antes** | `# Título {size=5xl color=coral}` | O bloco inteiro: título, parágrafo, item de lista, imagem ou bloco `:::` |
| Colado em um trecho entre colchetes, **sem espaço** | `Mas [ainda não]{color=coral} estava pronto` | Apenas o trecho |

```ribalta preview
# Resultados {color=accent}
O projeto ficou [pronto antes do prazo]{weight=bold color=green}.

Custos dentro do orçamento. {.muted}
```

Em um parágrafo de várias linhas, os atributos na última linha valem para o parágrafo todo. Uma linha com atributos sempre termina o parágrafo.

Atributos que são só um nome, como `italic`, `uppercase`, `border`, `shadow` e `wrap`, são ativados apenas por aparecerem. Para desativar um deles, use `=false`, como em `italic=false`.

:::tip
Para escrever uma chave literal no texto, use `\{`.
:::

## Cores

`color` muda a cor do texto e `bg` muda o fundo. As cores aceitam três formatos:

| Formato | Exemplo | Descrição |
| --- | --- | --- |
| Nome do tema | `color=accent` | Uma cor definida pelo tema ou por `@color`. |
| Hexadecimal | `color=#23976a` | `#rgb`, `#rrggbb` ou `#rrggbbaa`. |
| Com opacidade | `bg=coral/40` | Qualquer cor seguida de `/` e um número de 0 a 100. |

Todos os temas têm estas cores:

| Nome | Uso |
| --- | --- |
| `bg` | Fundo das páginas |
| `text` | Texto principal |
| `muted` | Texto secundário |
| `accent` | Cor de destaque do tema |
| `surface` | Fundo de cartões |
| `line` | Linhas, bordas e divisores |
| `coral`, `blue`, `green`, `yellow`, `violet` | Cores de apoio |
| `white`, `black`, `ink`, `paper`, `transparent` | Neutros |

Os valores exatos de cada tema estão em [Temas](/reference/themes/).

### Gradientes

Em `bg`, duas cores separadas por `..` criam um gradiente diagonal:

```ribalta preview
--- {bg=ink..violet color=white}
# Gradientes e transparência
Texto com 70% de opacidade {color=white/70}

[Coral]{.coral}, [azul]{color=blue}, [hexadecimal]{color=#23976a} e [fundo translúcido]{bg=yellow/30}
```

## Tamanhos e espaçamentos

Tamanhos e espaços usam escalas com nomes. Você também pode informar um número em pixels, como `size=72`.

### Tamanho do texto (`size`)

| Nome | Pixels | Nome | Pixels |
| --- | --- | --- | --- |
| `xs` | 20 | `2xl` | 64 |
| `sm` | 26 | `3xl` | 88 |
| `md` | 32 | `4xl` | 116 |
| `lg` | 40 | `5xl` | 150 |
| `xl` | 52 | `6xl` | 200 |
| | | `7xl` | 260 |

Em pixels, `size` aceita de 8 a 400.

### Espaçamento (`pad` e `gap`)

`pad` é a margem interna de um bloco e `gap` é o espaço entre os itens dele. `pad` aceita um valor, ou dois entre aspas (vertical e horizontal), como `pad="sm lg"`.

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

### Cantos arredondados (`radius`)

| Nome | Pixels |
| --- | --- |
| `none` | 0 |
| `sm` | 8 |
| `md` | 16 |
| `lg` | 28 |
| `xl` | 44 |
| `full` | Totalmente arredondado |

## Tipografia

| Atributo | Valores |
| --- | --- |
| `font` | Uma fonte do tema: `body`, `display`, `mono` ou uma declarada com `@font` |
| `weight` | `thin`, `light`, `regular`, `medium`, `semibold`, `bold`, `extrabold`, `black`, ou de 100 a 900 |
| `italic` | Itálico |
| `uppercase` | Caixa alta |
| `leading` | Altura da linha: `tight`, `normal`, `loose` ou um número de 0.6 a 3 |
| `tracking` | Espaço entre letras: `tight`, `normal`, `wide` ou um número em em, de -0.2 a 1 |

```ribalta preview
Novidades {.kicker}

# Tipografia com atributos {font=display}

Texto leve e espaçado {weight=light tracking=wide}

Texto em itálico e caixa alta {italic uppercase size=md}

Entrelinha folgada para textos que ocupam
mais de uma linha na página. {leading=loose size=md}
```


## Texto que encolhe

Com `shrink`, o texto começa no tamanho dele e encolhe até caber na página e nos blocos em volta. É útil quando o texto vem de uma variável e pode ser curto ou longo, como uma pergunta, um nome ou um título.

```ribalta
# $pergunta {size=5xl shrink=2xl}
```

- Sem valor, o texto pode encolher até `xs`. Com um valor, como `shrink=2xl`, ele nunca fica menor que esse tamanho.
- O tamanho de partida é o `size` do elemento ou, nos títulos, o tamanho do tema.
- Todos os textos com `shrink` da mesma página encolhem juntos, na mesma proporção, até a página caber. Cada um respeita o próprio mínimo.
- Se nem no mínimo o texto couber, ele fica no mínimo e o resto é cortado, como qualquer conteúdo que não cabe numa página de altura fixa.
- `shrink=false` desliga o encolhimento herdado de um estilo.
- O encolhimento é medido enquanto o documento é mostrado. Em HTML gerado no servidor, sem navegador, o texto aparece no tamanho de partida.

## Bordas, sombras e opacidade

| Atributo | Valores |
| --- | --- |
| `border` | Sozinho, aplica uma borda de 3 pixels na cor `line`. Aceita espessura e cor, como `border="2 coral"`. |
| `shadow` | `sm`, `md`, `lg` ou `none`. Sozinho, equivale a `md`. |
| `opacity` | De 0 a 1, ou porcentagem, como `opacity=60%`. |

```ribalta preview
::: row {gap=lg}
::: box {border pad=md radius=md}
Borda padrão
:::
::: box {border="2 coral" pad=md radius=lg}
Borda coral
:::
::: box {shadow=lg bg=surface pad=md radius=xl}
Sombra grande
:::
::: box {opacity=50% bg=surface pad=md}
Transparente
:::
:::
```

## Imagem de fundo

`image` coloca uma imagem de fundo que cobre todo o bloco ou a página. O endereço precisa começar com `https://` ou ser uma imagem guardada (`asset:`):

```ribalta preview
--- {image=https://images.unsplash.com/photo-1501785888041-af3ef285b470?w=1200 color=white}
::: box {bg=ink/60 pad=lg radius=lg}
# Expedição 2026
Planejamento da viagem de integração.
:::
```

## Estilos reutilizáveis

Quando a mesma combinação de atributos se repete, dê um nome a ela com `@style` no início do documento. Depois, aplique o estilo com um ponto, como `{.destaque}`:

```ribalta preview
@color marca #1e83d6
@style destaque {color=marca weight=bold}
@style selo {.card pad="sm md" bg=marca/15 color=marca}

# Proposta comercial
O prazo de entrega é de [dez dias úteis]{.destaque}.

Oferta válida até 30/11 {.selo}
```

Um estilo pode incluir outros estilos, como `.card` dentro de `.selo`. Quando você combina um estilo com atributos no mesmo lugar, os atributos escritos ali vencem: em `{.selo bg=yellow}`, o fundo fica amarelo.

Nomes de estilo usam letras minúsculas, números e hífens.

## Estilos embutidos

Todo tema já traz estes estilos:

| Estilo | Equivale a |
| --- | --- |
| `.muted` | `color=muted` |
| `.accent` | `color=accent` |
| `.lead` | `size=xl color=muted` |
| `.kicker` | `size=sm weight=bold uppercase tracking=0.14 color=accent` |
| `.display` | `font=display` |
| `.huge` | `font=display size=6xl leading=0.95` |
| `.card` | `bg=surface radius=lg pad=lg shadow=md` |
| `.dark` | `bg=ink color=paper` |
| `.center` | `align=center` |

```ribalta preview
--- {.dark .center}
Estilos embutidos {.kicker}

# Simples e consistente {.display}

Um texto de apoio com destaque. {.lead}

---
# 98% {.huge .accent}
de satisfação dos clientes {.muted}

::: box {.card}
Um cartão com fundo, cantos arredondados e sombra.
:::
```

## Cores como estilos

Toda cor do tema também funciona como estilo. `{.coral}` é o mesmo que `{color=coral}`, e isso vale para as cores que você cria com `@color`:

```ribalta preview
@color marca #1e83d6

# Status dos projetos
[Concluído]{.green} · [Em andamento]{.yellow} · [Atrasado]{.coral}

Responsável: equipe de produto {.marca}
```

## Próximos passos

- [Temas](/guides/themes/): escolha um tema e defina cores e fontes próprias.
- [Componentes](/guides/components/): combine estilos e layouts em blocos reutilizáveis.
- [Referência de atributos](/reference/attributes/): todos os atributos e onde cada um vale.
