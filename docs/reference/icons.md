---
title: Ícones
description: Como usar os ícones embutidos e a lista completa de nomes disponíveis.
---

O bloco `icon` desenha um ícone da biblioteca embutida, numa linha só:

```ribalta
:: icon nome {size=… color=…}
```

| Parte | Valores | Padrão |
| --- | --- | --- |
| `nome` | Um dos nomes da lista abaixo | Obrigatório ([R114](/docs/reference/errors/)) |
| `size` | Escala `xs`…`7xl` ou pixels de 8 a 400 ([Tamanhos](/docs/reference/attributes/#tamanhos)) | 96 px |
| `color` | Qualquer [cor](/docs/reference/attributes/#cores) | Cor do texto |

Ícones também aceitam os demais [atributos de estilo](/docs/reference/attributes/), como `opacity` e `step`. Um nome fora da lista é erro ([R115](/docs/reference/errors/)), com sugestão do nome mais parecido. Editores com suporte à Ribalta sugerem os nomes depois de `:: icon `.

```ribalta preview
::: grid {cols=3 gap=lg}
::: box {.card align=center}
:: icon rocket {size=3xl color=accent}
### Lançamento
:::
::: box {.card align=center}
:: icon users {size=3xl color=blue}
### Equipe
:::
::: box {.card align=center}
:: icon trending-up {size=3xl color=green}
### Crescimento
:::
:::
```

## Lista de ícones

```ribalta-icons
```
