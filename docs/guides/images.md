---
title: Imagens
description: Use imagens guardadas ou endereços externos, ajuste o enquadramento e use imagens de fundo.
---

Uma imagem na Ribalta é escrita como no Markdown: `![descrição](endereço)`. O endereço pode ser uma imagem guardada pelo programa em que você escreve ou um link `https://`.

## Imagens guardadas

Uma imagem guardada é referenciada por `asset:` seguido do identificador dela. O programa em que você escreve cuida de guardar o arquivo e de escrever essa referência para você:

```ribalta
![grafico-vendas](asset:3f2b8c1e-7a4d-4e9b-9c2a-1d5e6f7a8b9c)
```

Troque a descrição entre colchetes por um texto que descreva a imagem. Para usar a mesma imagem em outro documento, copie a linha `![…](asset:…)`.

## Imagens externas

Também é possível usar uma imagem publicada na internet, com um endereço que comece com `https://`:

```ribalta preview
# Nosso escritório
![Montanhas ao amanhecer](https://images.unsplash.com/photo-1501785888041-af3ef285b470?w=1200){h=640 radius=lg}
```

:::caution
Uma imagem externa depende do site que a publica. Se ela for removida ou mudar de endereço, deixa de aparecer no documento. Para imagens importantes, prefira guardar o arquivo.
:::

## Ajustando a imagem

Uma imagem sozinha numa linha é um bloco. Os atributos vão entre chaves, logo depois do endereço:

| Atributo | O que faz |
| --- | --- |
| `fit=contain` | Mostra a imagem inteira dentro da área. É o padrão. |
| `fit=cover` | Preenche toda a área, cortando as bordas se for preciso. |
| `w`, `h` | Largura e altura, em pixels (`480`), porcentagem (`50%`) ou `full`. |
| `radius` | Arredonda os cantos: `sm`, `md`, `lg` ou pixels. |
| `shadow` | Sombra: `sm`, `md`, `lg` ou `none`. Sozinho, vale `md`. |
| `border` | Borda, como `border="4 accent"`. |
| `opacity` | Transparência, de `0` a `1` ou em porcentagem. |

O `fit` só faz diferença quando a área tem proporção diferente da imagem, por exemplo quando você define `w` e `h` ao mesmo tempo:

```ribalta preview
::: columns {gap=lg}
![Paisagem inteira](https://images.unsplash.com/photo-1501785888041-af3ef285b470?w=1200){fit=contain h=560}
![Paisagem recortada](https://images.unsplash.com/photo-1501785888041-af3ef285b470?w=1200){fit=cover h=560 radius=lg shadow}
:::
```

Para colocar texto ao lado da imagem, use colunas. Veja [Layouts](/guides/layout/).

## Imagens no meio do texto

Uma imagem dentro de um parágrafo acompanha o texto e tem a altura da letra. Os atributos vão colados, sem espaço:

```ribalta preview
# Parceiros
Apoio: ![logotipo](https://images.unsplash.com/photo-1501785888041-af3ef285b470?w=1200){radius=sm} Fundação Horizonte {size=2xl}
```

Para mudar o tamanho, mude o `size` do parágrafo ou use `h` na imagem.

## Imagens de fundo

O atributo `image` coloca uma imagem no fundo de uma página ou de um bloco. Ela cobre toda a área e fica centralizada:

```ribalta preview
--- {image=https://images.unsplash.com/photo-1501785888041-af3ef285b470?w=1200 color=white align=center}
::: box {bg=ink/60 pad=xl radius=lg}
# Planejamento 2027 {size=5xl}
Encontro anual de liderança {.lead}
:::
```

`image` aceita os mesmos endereços que as imagens comuns: `https://` ou uma imagem guardada (`image=asset:…`).

```ribalta
--- {image=asset:3f2b8c1e-7a4d-4e9b-9c2a-1d5e6f7a8b9c}
::: box {image=asset:3f2b8c1e-7a4d-4e9b-9c2a-1d5e6f7a8b9c h=400 radius=lg}
:::
```

:::tip
Texto sobre uma foto costuma ficar difícil de ler. Coloque-o num `box` com fundo semitransparente, como `bg=ink/60`, ou escolha uma imagem escura e use `color=white`.
:::

## Erros comuns

| Código | Quando acontece |
| --- | --- |
| R301 | Endereço não permitido: não começa com `https://` nem é uma imagem guardada (`asset:` seguido do identificador). |
| R301 | `fit` com um valor diferente de `cover` ou `contain`. |
| R209 | Atributo desconhecido na imagem, como `fitt`. |
