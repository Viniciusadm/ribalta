---
title: Temas
description: Escolha um tema pronto, ajuste cores e fontes e compartilhe definições entre vários documentos.
---

O tema define a aparência básica do documento: cores, fontes, tamanhos dos títulos e margens. Você escolhe um tema pronto e ajusta o que quiser com definições no início do texto.

## Escolhendo um tema

Use `@theme` no início do documento, antes da primeira página. Sem `@theme`, o documento usa o tema `paper`.

```ribalta
@theme editorial
```

A Ribalta tem três temas prontos.

### paper

Fundo creme, títulos marcantes em Archivo Black, texto em Barlow e destaque coral. É o tema padrão.

```ribalta preview
@theme paper

Relatório anual {.kicker}
# Resultados de 2026
Receita, clientes e próximos passos. {.lead}
```

### ink

Fundo escuro, tipografia Inter em todos os textos e destaque âmbar. Bom para auditórios e telas grandes.

```ribalta preview
@theme ink

Relatório anual {.kicker}
# Resultados de 2026
Receita, clientes e próximos passos. {.lead}
```

### editorial

Fundo claro e quente, títulos com serifa em Fraunces, texto em Source Sans 3, destaque vermelho escuro e margens mais generosas.

```ribalta preview
@theme editorial

Relatório anual {.kicker}
# Resultados de 2026
Receita, clientes e próximos passos. {.lead}
```

A lista completa de cores e fontes de cada tema está em [Referência de temas](/reference/themes/).

## Cores

`@color nome valor` cria uma cor nova ou substitui uma cor do tema. O valor pode ser um hexadecimal, outra cor do tema ou uma cor com opacidade, como `coral/80`:

```ribalta preview
@theme ink
@color accent #4fd1c5
@color marca #7c5cd6

Novo produto {.kicker}
# Painel de vendas {color=accent}
Lançamento previsto para o segundo semestre. {.marca}
```

Substituir `accent` muda todos os lugares que usam a cor de destaque, como o estilo `.kicker`. Uma cor nova, como `marca`, pode ser usada em qualquer atributo de cor (`color=marca`, `bg=marca/20`) e também como estilo (`{.marca}`).

Nomes de cor usam letras minúsculas, números e hífens.

## Fontes

`@font nome "Família" pesos` declara uma fonte do Google Fonts. Escreva o nome da família entre aspas e, depois, os pesos que você vai usar, de 100 a 900:

```ribalta preview
@font titulo "Bebas Neue" 400
@font body "Inter" 400 700

# Plano de expansão {font=titulo size=5xl}
Três mercados novos até o fim do ano.
```

Uma fonte nova, como `titulo`, é aplicada com `font=titulo`. Usar um nome que já existe substitui a fonte do tema:

| Nome | Onde é usada |
| --- | --- |
| `body` | Texto comum e títulos menores |
| `display` | Títulos principais e os estilos `.display` e `.huge` |
| `mono` | Código |

:::note
Declare apenas os pesos que você realmente usa. Cada peso é mais um arquivo para carregar.
:::

## Tema base

O tema base reúne definições compartilhadas por vários documentos. É o lugar certo para a identidade visual de uma empresa, de um curso ou de um cliente.

O tema base é um texto à parte, escrito com as mesmas definições do início de um documento. O programa que compila o documento junta os dois: primeiro o tema base, depois o documento.

```ribalta
@theme editorial
@color marca #1e83d6
@font titulo "Bebas Neue" 400
@style rotulo {size=sm weight=bold uppercase tracking=0.14 color=marca}
```

Regras do tema base:

- Ele só pode ter definições, como `@theme`, `@page`, `@color`, `@font`, `@style` e `@component`. Páginas no tema base são um erro.
- `@title` e `@author` são ignorados ali, com um aviso, porque pertencem a cada documento.
- Cada documento pode sobrescrever o que quiser: o próprio `@theme`, o tamanho das páginas, uma cor, uma fonte, um estilo ou um componente com o mesmo nome.
- Erros no tema base são apontados no próprio tema, nunca nos documentos.

## Próximos passos

- [Estilos](/guides/styling/): use as cores e fontes do tema nas suas páginas.
- [Componentes](/guides/components/): coloque componentes no tema base para reutilizá-los em todos os documentos.
- [Referência de temas](/reference/themes/): todas as cores, fontes e tamanhos de cada tema.
