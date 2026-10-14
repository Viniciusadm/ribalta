---
title: Tamanhos e documentos
description: Escolha o tamanho das páginas com @page, use altura automática para documentos longos e mude o tamanho de uma página só.
---

Um documento Ribalta é uma sequência de páginas. Cada linha `---` começa uma página nova, e cada página tem uma largura e uma altura em pixels. Uma apresentação é um documento de páginas 1920 × 1080, uma por slide. Um relatório, um artigo ou uma tela comprida pode ser uma página só, com largura fixa e altura que cresce com o conteúdo.

## O que é uma página

Uma página é a área onde o conteúdo é desenhado. Tudo o que você escreve entre dois `---` fica na mesma página:

```ribalta
# Primeira página
Conteúdo da primeira página.

---
# Segunda página
Conteúdo da segunda página.
```

As medidas da linguagem são pixels da página, qualquer que seja a tela em que ela aparece: `w=480` é sempre 480 pixels de uma página, e `x` e `y` contam a partir do canto superior esquerdo dela.

## Tamanho padrão

O programa que compila o documento escolhe o tamanho padrão das páginas. Sem um padrão do programa, a página tem 1920 pixels de largura e altura automática.

Para não depender do padrão, diga o tamanho no próprio documento com `@page`.

## Escolhendo o tamanho com `@page`

Escreva `@page` no início do documento, junto com `@title` e `@theme`, com a largura e a altura separadas por `x`:

```ribalta preview
@page 1440x1080

# Página 4:3
Esta página tem 1440 × 1080 pixels.
```

| Você escreve | Tamanho da página |
| --- | --- |
| `@page 1920x1080` | 1920 × 1080, o formato 16:9 das apresentações |
| `@page 1440x1080` | 1440 × 1080, o formato 4:3 |
| `@page 1080x1920` | 1080 × 1920, uma página vertical, como a tela de um celular |
| `@page 800xauto` | 800 de largura e altura automática |

Também dá para nomear cada medida com `width` e `height`. Nessa forma, `@page` só muda o que nomeia, e o resto continua como estava:

```ribalta
@page width=800 height=auto

# Relatório mensal
```

```ribalta
@page height=auto

# Mesma largura do padrão, altura automática
```

Cada medida vai de 16 a 16384 pixels, e a altura também aceita `auto`. Um valor fora dessas regras, como `@page 16:9`, é erro ([R515](/reference/errors/)).

## Altura automática

Com `height=auto`, a página tem largura fixa e cresce para caber todo o conteúdo. É o formato de um documento inteiro numa página só, de uma tela comprida que se rola, de um artigo ou de um relatório:

```ribalta
@title Relatório de vendas
@page 800xauto

# Relatório de vendas
Resumo do trimestre, região por região.

## Sul
A receita cresceu 12%, puxada pelos clientes novos.

## Sudeste
A receita ficou estável, com mais retenção.

## Norte
A receita caiu 4%. O plano de recuperação começa em março.

| Região | Receita |
| --- | ---: |
| Sul | R$ 4,2 mi |
| Sudeste | R$ 9,8 mi |
| Norte | R$ 1,1 mi |
```

O que muda numa página de altura automática:

- **O conteúdo cresce.** A altura da página é a altura do conteúdo, mais a margem do tema. Escreva quanto precisar.
- **Nada é cortado.** Numa página de altura fixa, o que passa da borda de baixo fica de fora; aqui a página aumenta.
- **`valign` não tem espaço para agir.** Como a página tem exatamente a altura do conteúdo, não sobra espaço vertical para centralizar ou empurrar o conteúdo para baixo.

## Uma página com outro tamanho

`width` e `height` no separador `---` mudam o tamanho só daquela página. Assim, um documento horizontal pode ter uma página vertical no meio:

```ribalta
@page 1920x1080

# Campanha de verão

--- {width=1080 height=1920 align=center}
# Story
A versão vertical, para celular.

---
# Próximos passos
```

A terceira página volta ao tamanho do `@page`. Para deixar uma página só com altura automática, use `--- {height=auto}`:

```ribalta
# Resumo

--- {height=auto}
# Apêndice
Uma lista longa de detalhes, que cresce o quanto precisar.
```

Valores inválidos no separador também são erro [R515](/reference/errors/), com as mensagens "largura de página inválida" ou "altura de página inválida".

## Quem decide o tamanho

Cada página começa com o tamanho padrão e pode ser ajustada em quatro níveis. Cada nível vence o anterior:

1. O padrão do programa que compila o documento.
2. `@page` no [tema base](/guides/themes/#tema-base), que vale para todos os documentos que usam esse tema.
3. `@page` no documento.
4. `width` e `height` no separador da página.

Por exemplo, um tema base com `@page 1440x1080` deixa todos os documentos em 4:3; um documento com `@page 1920x1080` volta a 16:9 só para ele; e uma página com `--- {height=auto}` fica com 1920 de largura e altura automática.

## Número da página

Dentro de `@header`, `@footer` e `@overlay`, `$page` é o número desta página e `$pages` é o total de páginas do documento:

```ribalta preview
@footer {align=right}
Página $page de $pages
@end

# Abertura

---
# Resultados

---
# Próximos passos
```

A capa conta como página 1, mesmo sem rodapé. Veja [Cabeçalho, rodapé e camada livre](/guides/header-footer/#campos-automáticos).

## Erros comuns

| Código | Quando acontece |
| --- | --- |
| [R515](/reference/errors/) | `@page` sem largura e altura válidas, como `@page 16:9`, ou `width`/`height` inválidos no separador. |
| [R501](/reference/errors/) | `@page` sem valor. |
| [R103](/reference/errors/) | `@page` depois da primeira página. |
