---
title: Cabeçalho, rodapé e camada livre
description: Escreva uma vez e repita o mesmo conteúdo, no mesmo lugar, em todas as páginas.
---

Logo da empresa, nome do evento, "Confidencial", número da página: em vez de repetir em cada página, escreva uma vez com `@header` e `@footer`. A Ribalta coloca esse conteúdo no mesmo lugar em todas as páginas, **menos na capa**.

Para o que precisa ficar solto, por cima do conteúdo (logo num canto, marca d'água, selo "Rascunho"), use a [camada livre](#camada-livre-overlay), `@overlay`.

## O básico

Escreva o bloco no início do documento, junto com `@title` e `@theme`, e feche com `@end`:

```ribalta preview
@title Plano 2027
@theme paper

@footer
Plano 2027 · Confidencial
@end

# Plano 2027

---
# Onde estamos

---
# Para onde vamos
```

A primeira página é a capa e fica sem rodapé. Nas outras, o rodapé aparece na parte de baixo, dentro da margem da página.

O conteúdo entre `@footer` e `@end` é escrito com a mesma linguagem das páginas: texto, imagens, ícones, estilos, componentes e blocos como `::: columns`.

## Esquerda e direita

Para pôr uma parte de cada lado, use `::: columns`. Cada item vira uma coluna, então separe os itens com uma **linha em branco**:

```ribalta preview
@title Pitch Lumen
@theme ink

@header
::: columns {valign=center}
:: icon rocket {size=lg color=accent}

Lumen Analytics {align=right}
:::
@end

@footer
::: columns
Confidencial

$page / $pages {align=right}
:::
@end

# Lumen Analytics

---
# Tração
```

:::tip
Duas linhas seguidas, sem linha em branco entre elas, formam um único parágrafo com quebra de linha. Por isso elas ficariam juntas, na mesma coluna.
:::

## Campos automáticos

Dentro de `@header`, `@footer` e `@overlay`, estes campos são preenchidos sozinhos em cada página:

| Campo | Vira |
| --- | --- |
| `$page` | o número desta página (a capa conta como 1) |
| `$pages` | o total de páginas do documento |
| `$title` | o `@title` do documento |
| `$author` | o `@author` do documento |
| `$date` | a data de hoje, no idioma de quem está vendo |

Um campo com nome errado aparece como erro, com a sugestão do mais parecido: `$pagina` sugere `$page`.

## Aparência

Por padrão, cabeçalho e rodapé usam texto pequeno (`size=sm`) na cor `muted` do tema. Numa página escura, a cor acompanha a página.

Para mudar o padrão, coloque atributos na linha do `@header` ou `@footer`. Eles valem para o bloco inteiro:

```ribalta preview
@theme paper

@footer {align=center color=accent weight=bold}
Semana de Produto 2026
@end

# Abertura

---
# Agenda
```

Atributos dentro do bloco continuam funcionando normalmente, como `{align=right}` num parágrafo.

## Camada livre (`@overlay`)

`@overlay` funciona como o cabeçalho e o rodapé (escrito uma vez, some na capa), mas não reserva faixa: é uma camada do tamanho da página, **por cima** do conteúdo. Serve para logo num canto, marca d'água ou selo "Rascunho".

```ribalta preview
@title Plano 2027
@theme paper

@overlay
Rascunho {x=1500 y=40 color=coral weight=bold uppercase}
@end

# Plano 2027

---
# Onde estamos

O selo fica por cima do conteúdo, no canto.
```

Dentro da camada, um bloco com `x` e `y` é posicionado a partir do canto de cima à esquerda **da página**, sem contar a margem: `x=0 y=0` é o canto exato.

Sem `x` e `y`, o conteúdo fica dentro da margem da página. Use `align` e `valign` na linha do `@overlay` para escolher o canto, sem calcular coordenadas:

```ribalta preview
@theme ink

@overlay {align=right valign=bottom}
:: icon sparkles {size=2xl color=accent}
@end

# Abertura

---
# Agenda
```

Diferente do cabeçalho e do rodapé, a camada livre não tem aparência padrão: o texto usa o tamanho e a cor normais da página. Para uma marca d'água, use `opacity`, como em `{opacity=0.1}`.

## Capa e exceções

A capa fica sem cabeçalho, rodapé e camada livre. Por padrão, a capa é a primeira página. Na linha `---` de cada página, você pode mudar isso:

| Na linha `---` | Efeito |
| --- | --- |
| `{cover}` | a página também vira capa, sem cabeçalho, rodapé e camada livre (bom para páginas de seção) |
| `{cover=false}` | a primeira página deixa de ser capa |
| `{footer=off}` | tira o rodapé só desta página |
| `{header=off}` | tira o cabeçalho só desta página |
| `{overlay=off}` | tira a camada livre só desta página |
| `{footer=on}` | mostra o rodapé mesmo numa capa |

```ribalta
--- {cover}
# Parte 2: Resultados

--- {footer=off}
![Foto do time](https://images.unsplash.com/photo-1522071820081-009f0129c71c?w=1600){fit=cover w=100% h=100%}
```

## No tema do projeto

Um `@header`, `@footer` ou `@overlay` escrito no [tema do projeto](/docs/guides/themes/#tema-do-projeto) vale para todos os documentos do projeto. É o lugar do rodapé padrão da empresa.

- Se o documento tiver o seu próprio `@footer`, ele **substitui** o do tema.
- Para tirar o rodapé do tema num documento, escreva `@footer none`. O mesmo vale para `@header none` e `@overlay none`.

```ribalta
@title Documento sem o rodapé da empresa
@footer none
```

## Por dentro

Cabeçalho e rodapé ocupam uma faixa reservada: o conteúdo da página nunca fica por baixo deles. Um rodapé de uma linha cabe dentro da margem do tema e não muda o layout. Se o bloco for maior, o espaço do conteúdo diminui. A camada livre não ocupa espaço: o layout da página é o mesmo com ou sem ela.

Algumas coisas não valem dentro de `@header`, `@footer` e `@overlay`:

- **Revelação passo a passo.** Esses blocos aparecem sempre inteiros; `step` e `::: steps` são ignorados, com um aviso.
- **Notas.** `::: notes` dentro do bloco é ignorado, com um aviso. Escreva as notas na própria página.
