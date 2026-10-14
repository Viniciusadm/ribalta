---
title: Componentes
description: Crie blocos reutilizáveis com parâmetros, conteúdo e espaços nomeados para manter as páginas consistentes.
---

Um componente é um trecho de página com nome, que você define uma vez e usa quantas vezes quiser. Ele recebe parâmetros, como um molde: muda o texto, o visual continua o mesmo.

Dentro de um componente valem os mesmos recursos de qualquer página, inclusive `::: if`, `::: for` e expressões sobre os parâmetros. Veja [Variáveis, dados e repetição](/guides/data/#componentes-que-recebem-dados).

## Definindo um componente

Escreva a definição no início do documento, junto com `@title`, `@theme` e as outras definições, antes da primeira página:

```ribalta
@component metrica {valor rotulo cor=accent}
::: box {.card align=center}
$valor {size=5xl weight=bold color=$cor}
$rotulo {.muted}
:::
@end
```

- `@component` é seguido do nome e, entre chaves, da lista de parâmetros.
- Tudo entre essa linha e `@end` é o corpo do componente, escrito com a mesma linguagem das páginas.
- O nome usa letras minúsculas, números e hífens, como `metrica` ou `cartao-equipe`.

:::note
Os nomes dos blocos da linguagem (`box`, `columns`, `steps`, `chart` e os demais) são reservados e não podem ser usados como nome de componente.
:::

### Parâmetros obrigatórios e com valor padrão

| Forma | Significado |
| --- | --- |
| `valor` | Parâmetro obrigatório. Usar o componente sem ele é erro. |
| `cor=accent` | Parâmetro com valor padrão. Se você não informar, vale `accent`. |
| `titulo="Sem título"` | Valor padrão com espaços, entre aspas. |

### Usando os parâmetros

Dentro do corpo, `$nome` é trocado pelo valor do parâmetro. Isso funciona em qualquer lugar:

- no texto: `## $titulo` ou `Responsável: $nome`;
- em valores de atributos: `{color=$cor}` ou `{bg=$fundo}`;
- numa linha sozinha: `$rotulo` vira um parágrafo com o valor.

O valor pode ter formatação de texto, como `**negrito**`, e ela é aplicada na página.

## Usando um componente

Há duas formas de usar um componente, as mesmas dos blocos da linguagem:

- **Uma linha só**, com `::`, quando ele não tem conteúdo: `:: metrica {valor=38% rotulo="crescimento"}`.
- **Contêiner**, com `:::` e um `:::` de fechamento, quando ele recebe conteúdo (veja a seção seguinte).

Os parâmetros vão entre chaves, sempre com nome. Valores com espaços ficam entre aspas.

```ribalta preview
@component metrica {valor rotulo cor=accent}
::: box {.card align=center}
$valor {size=5xl weight=bold color=$cor}
$rotulo {.muted}
:::
@end

# Resultados do trimestre
::: grid {cols=3 gap=lg}
:: metrica {valor=38% rotulo="crescimento da receita"}
:: metrica {valor=94 rotulo="novos clientes" cor=green}
:: metrica {valor=4,8 rotulo="nota de satisfação" cor=violet}
:::
```

## Conteúdo com `$content`

Quando você usa o componente como contêiner, tudo o que estiver entre a abertura e o fechamento vai para `$content`. Coloque `$content` numa linha sozinha, no ponto do corpo em que o conteúdo deve aparecer:

```ribalta preview
@component destaque {titulo}
::: box {.card gap=sm}
## $titulo
$content
:::
@end

::: destaque {titulo="Próximos passos"}
- Revisar o orçamento com a diretoria
- Contratar duas pessoas para o suporte
- Lançar o novo plano em março
:::
```

O conteúdo pode ter qualquer coisa: texto, listas, imagens, outros blocos e outros componentes.

:::caution
Se o componente não tiver `$content` no corpo, o conteúdo que você passar é ignorado e o compilador avisa.
:::

## Espaços nomeados

Às vezes um componente precisa de mais de uma área de conteúdo, como as duas colunas de uma comparação. Para isso existem os espaços nomeados:

1. No corpo do componente, escreva `$nome-do-espaco` numa linha sozinha, com um nome que **não** seja um parâmetro.
2. Ao usar o componente, preencha o espaço com um bloco `::: slot nome-do-espaco` … `:::`.
3. O restante do conteúdo continua indo para `$content`.

```ribalta preview
@component comparacao {antes depois}
::: columns {gap=xl}
::: box {.card gap=sm}
## $antes
$content
:::
::: box {.card gap=sm bg=accent color=white}
## $depois
$resultado
:::
:::
@end

::: comparacao {antes="Planilhas soltas" depois="Um painel único"}
Cada área exporta, cola e formata os números à mão.
::: slot resultado
Os dados chegam sozinhos e o relatório se atualiza a cada hora.
:::
:::
```

Espaços que você não preencher ficam vazios, sem erro. Preencher um espaço que o componente não tem é erro, e a mensagem sugere o nome mais parecido.

## Componentes dentro de componentes

O corpo de um componente pode usar outros componentes, e o conteúdo passado a um componente também:

```ribalta preview
@component selo {texto}
::: box {bg=accent color=white pad=sm radius=lg}
$texto {size=sm weight=bold uppercase}
:::
@end

@component plano {nome preco}
::: box {.card gap=sm}
:: selo {texto=$nome}
## $preco
$content
:::
@end

::: grid {cols=2 gap=lg}
::: plano {nome=Essencial preco="R$ 49"}
Até 5 usuários
:::
::: plano {nome=Equipe preco="R$ 149"}
Usuários ilimitados
:::
:::
```

Um componente não pode usar a si mesmo, nem direta nem indiretamente (por exemplo, `a` usa `b` e `b` usa `a`). A Ribalta detecta o ciclo e mostra um erro. Componentes podem ser aninhados até 16 níveis.

## Revelando um componente por etapas

O atributo `step` também vale na chamada do componente. O componente inteiro aparece na etapa indicada:

```ribalta
:: metrica {valor=38% rotulo="crescimento" step=1}
:: metrica {valor=94 rotulo="novos clientes" step=2}
```

Veja mais em [Revelação por etapas](/guides/reveal/).

## Estilizando um componente

Componentes recebem apenas parâmetros. Estilos como `.card` e `#id` na chamada são erro. Para mudar a aparência de um uso específico, envolva o componente num `box`:

```ribalta
::: box {.dark pad=lg}
:: metrica {valor=38% rotulo="crescimento"}
:::
```

Se a mudança deve valer sempre, prefira criar um parâmetro, como o `cor=accent` do exemplo `metrica`.

:::tip
Para usar os mesmos componentes em vários documentos, defina-os no tema base. Veja [Temas](/guides/themes/).
:::

## Parâmetros com valores aceitos

Separe os valores aceitos com `|` no valor padrão. O primeiro é o padrão, e qualquer outro valor é erro ([R418](/reference/errors/)). Combine com um estilo por variante, como `.botao-$tipo`, para não repetir propriedades:

```ribalta preview
@style botao-primary {bg=accent color=white}
@style botao-ghost {color=accent border="2 accent"}
@component botao {texto tipo=primary|ghost}
::: row {.botao-$tipo pad="sm lg" radius=full}
**$texto**
:::
@end

::: row {gap=md}
:: botao {texto=Assinar}
:: botao {texto="Saber mais" tipo=ghost}
:::
```

## Registros como parâmetro

Um parâmetro pode receber um registro de `@data` inteiro: `:: cartao {p=$pessoa}`. No corpo, use `$p.nome`, `$p.cargo` e assim por diante. Na hora de usar o componente, `if` e `click` também são aceitos: `:: cartao {p=$pessoa if=$mostrar click="go #equipe"}`.

## Erros comuns

| Código | Quando acontece |
| --- | --- |
| R401 | `$content` foi usado fora de um `@component`. |
| R404 | O componente usa a si mesmo, direta ou indiretamente. |
| R405 | Componentes aninhados em mais de 16 níveis. |
| R406 | A chamada tem `.estilo` ou `#id`. Envolva o componente num `box`. |
| R407 | A chamada informa um parâmetro que o componente não tem. |
| R408 | A chamada tem texto solto em vez de parâmetros entre chaves. |
| R409 | Falta um parâmetro obrigatório. |
| R410 | O corpo usa `$nome`, mas `nome` não foi declarado como parâmetro. |
| R411 | Aviso: o componente recebeu conteúdo, mas não usa `$content`. |
| R412 | O nome do componente é o de um bloco da linguagem. |
| R413 | O mesmo componente foi definido duas vezes no documento. |
| R414 | `::: slot` preenche um espaço que o componente não tem. |
| R415 | `::: slot` sem o nome do espaço. |
| R416 | `::: slot` fora da chamada de um componente. |
| R417 | O mesmo espaço foi preenchido duas vezes. |
| R504 | `@component` sem o `@end` correspondente. |
| R511 | Nome de componente inválido (use letras minúsculas, números e hífens). |

Erros dentro do corpo de um componente aparecem na linha em que ele é **usado**, com a indicação "(no componente `nome`)". A lista completa está em [Mensagens de erro](/reference/errors/).
