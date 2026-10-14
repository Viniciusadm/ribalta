---
title: "Variáveis, dados e repetição"
description: "Use @let, @data, ::: for e ::: if para gerar páginas a partir de valores e listas, sem repetir código."
---

Quando a mesma informação aparece em vários lugares, ou quando uma página é feita de muitos itens parecidos, escrever tudo à mão dá trabalho e abre espaço para erros. A Ribalta resolve isso com quatro recursos que funcionam juntos:

| Recurso | Para quê |
| --- | --- |
| `@let` | Guardar um valor (texto, número, verdadeiro/falso, lista) e usá-lo em qualquer lugar com `$nome`. |
| `@data` | Declarar uma tabela de dados, como uma lista de pessoas, planos ou números. |
| `::: for` | Repetir um trecho para cada item de uma lista. |
| `::: if` | Mostrar um trecho só quando uma condição é verdadeira. |

Tudo isso é resolvido **antes** de mostrar o documento: o compilador gera as páginas finais e avisa sobre qualquer problema. Para conteúdo que muda **enquanto** o documento é mostrado (ao clicar, por exemplo), veja [Interatividade e navegação](/guides/interactive/).

## Variáveis com `@let`

Declare a variável no início do documento, junto com as outras definições, e use `$nome` no texto ou em atributos:

```ribalta preview
@let evento = "Encontro Anual 2026"
@let destaque = coral
@let espaco = lg

Convite {.kicker}
# $evento {color=$destaque}
::: row {gap=$espaco}
:: icon calendar {color=$destaque}
Inscrições até sexta
:::
```

- O nome usa letras minúsculas, números, hífens e sublinhados, começando por uma letra.
- O valor pode ser texto (`"com espaços"` entre aspas, ou uma palavra só, como `coral`), número (`42`, `3.5`), `true`/`false` ou uma lista (`[Ideia, Protótipo, Lançamento]`).
- Uma palavra sem `$` é sempre texto: `@let tema = escuro` guarda o texto `escuro`.
- `$nome` que não foi declarado continua como texto comum, então `US$ 10` e `$100` não mudam.

Para cores, `@color` continua sendo o melhor caminho. Use `@let` para o resto: tamanhos, espaçamentos, textos repetidos e opções que ligam ou desligam partes do documento.

### Uma variável controla muitos elementos

Combine `@let` com `@style` para mudar a aparência do documento inteiro trocando um único valor:

```ribalta preview
@let compacto = false
@let espaco = $compacto ? sm : lg
@let corpo = $compacto ? md : xl
@style cartao {.card pad=$espaco gap=$espaco}
@style texto {size=$corpo}

::: grid {cols=2 gap=$espaco}
::: box {.cartao}
## Rápido
Compila enquanto você digita. {.texto}
:::
::: box {.cartao}
## Seguro
Nada é executado no navegador. {.texto}
:::
:::
```

Troque `compacto` para `true` e todos os cartões ficam menores.

### Expressões

Dentro de `$( … )`, você pode calcular valores:

```ribalta preview
@let preco = 49
@let usuarios = 12

## Equipe de $usuarios pessoas
Total: **R$ $($preco * $usuarios)** por mês
Por ano, com 15% de desconto: R$ $(round($preco * $usuarios * 12 * 0.85))
```

| Operação | Exemplo |
| --- | --- |
| Aritmética | `$a + 1`, `$preco * 12`, `$total / 2`, `$n % 2` |
| Juntar texto | `"Olá, " + $nome` |
| Comparação | `$a == $b`, `$a != pro`, `$n > 10`, `$n <= 3` |
| Pertence à lista | `$fruta in [maçã, pera]`, `chave in $mochila` |
| Lógica | `$a and $b`, `$a or $b`, `not $a` |
| Escolha | `$compacto ? sm : lg` |
| Funções | `len($lista)`, `min($lista)`, `max($lista)`, `sum($lista)`, `round($x)`, `round($x, 1)`; em ações, `random(1, 6)` e `pick($lista)` ([Sorteios](/guides/interactive/#sorteios)) |

Não há funções criadas pelo usuário, laços infinitos nem acesso a nada fora do documento: as expressões só calculam valores.

## Dados com `@data`

Uma tabela entre `@data nome` e `@end` vira uma lista de registros. A primeira linha dá o nome dos campos:

```ribalta
@data equipe
| nome  | cargo      | cor   |
| ---   | ---        | ---   |
| Ana   | CTO        | blue  |
| Bruno | Design     | coral |
| Caio  | Engenharia | green |
@end
```

- Colunas só com números viram números; `true`/`false` (ou `sim`/`não`) viram verdadeiro/falso; o resto é texto.
- Cada linha é acessada com `$item.campo`, como `$pessoa.nome`.
- Um campo de uma lista inteira devolve a coluna: `max($vendas.total)` é o maior total.

## Repetição com `::: for`

`::: for item in $lista` repete o conteúdo para cada item. Os blocos gerados entram direto no bloco pai, então funcionam como células de uma grade, colunas ou etapas:

```ribalta preview
@data equipe
| nome  | cargo      | cor   |
| ---   | ---        | ---   |
| Ana   | CTO        | blue  |
| Bruno | Design     | coral |
| Caio  | Engenharia | green |
@end

## Nosso time
::: grid {cols=3 gap=lg}
::: for pessoa in $equipe
::: box {.card align=center}
:: icon user {size=3xl color=$pessoa.cor}
### $pessoa.nome
$pessoa.cargo {.muted}
:::
:::
:::
```

| Forma | Significado |
| --- | --- |
| `::: for x in $lista` | Repete para cada item. |
| `::: for x, i in $lista` | Também dá a posição `$i`, começando em 1. |
| `::: for i in 1..5` | Repete para os números de 1 a 5. |

Listas Markdown geradas por um `for` são juntadas numa lista só, com a numeração correta. Use a posição para revelar um item por vez:

```ribalta preview
@let etapas = [Descoberta, Protótipo, Piloto, Lançamento]

## Como chegamos lá
::: for etapa, i in $etapas
1. $etapa {step=$i}
:::
```

## Condições com `::: if`

`::: if condição` mostra o conteúdo só quando a condição é verdadeira. Logo depois, `::: else if` e `::: else` dão alternativas. Cada bloco fecha com `:::`, como qualquer outro:

```ribalta preview
@let publico = executivo

::: if $publico == tecnico
O compilador roda em **wasm**, direto no navegador.
:::
::: else if $publico == executivo
Reduzimos o tempo de montagem de slides em **70%**.
:::
::: else
Escreva texto, receba slides.
:::
```

Para um único elemento, use o atributo `if`:

```ribalta preview
@let rascunho = true

# Proposta comercial
Versão preliminar, sujeita a revisão {.muted if=$rascunho}
```

Assim como o `for`, o `if` não cria uma caixa a mais: o conteúdo escolhido entra no bloco pai.

## Gráficos e tabelas a partir de dados

`chart` e `table` aceitam `data=$lista`:

```ribalta preview
@data vendas
| mes | receita | custo |
| --- | ---     | ---   |
| Jan | 120     | 80    |
| Fev | 150     | 90    |
| Mar | 210     | 110   |
@end

::: columns {gap=xl}
:: chart {data=$vendas type=line}
:: table {data=$vendas columns="mes receita"}
:::
```

| Atributo | Onde | Significado | Padrão |
| --- | --- | --- | --- |
| `data` | `chart`, `table` | A lista de registros | (obrigatório) |
| `label` | `chart` | O campo com os rótulos | O primeiro campo de texto |
| `series` | `chart` | Os campos com os valores, separados por espaço | Todos os campos numéricos |
| `columns` | `table` | Os campos exibidos, em ordem | Todos |

## Diagramas guiados por dados

Não é preciso um bloco especial para desenhar a partir de dados: um `for` com caixas de largura calculada já resolve.

```ribalta preview
@data adocao
| mes | equipes |
| --- | ---     |
| Jan | 120     |
| Fev | 340     |
| Mar | 910     |
@end

## Equipes usando o produto
::: for m, i in $adocao
::: row {gap=md valign=center step=$i}
$m.mes {w=80 .muted}
::: box {w="$(round($m.equipes / max($adocao.equipes) * 100))%" h=40 bg=accent radius=sm}
:::
**$m.equipes**
:::
:::
```

## Componentes que recebem dados

Um parâmetro de componente pode receber um registro inteiro, e o corpo do componente pode usar `if` e `for` sobre os parâmetros:

```ribalta preview
@data planos
| id    | nome    | preco | usuarios |
| ---   | ---     | ---   | ---      |
| basic | Básico  | 0     | 3        |
| pro   | Pro     | 49    | 25       |
| team  | Equipes | 199   | 200      |
@end

@component plano {p tom=claro|escuro}
::: box {.card align=center .plano-$tom}
### $p.nome
R$ $p.preco {size=3xl weight=black}
::: if $p.usuarios >= 100
Suporte dedicado {.kicker}
:::
:::
@end
@style plano-claro {bg=surface}
@style plano-escuro {bg=ink color=paper}

::: grid {cols=3 gap=lg}
::: for p in $planos
:: plano {p=$p tom="$($p.id == pro ? escuro : claro)"}
:::
:::
```

- `tom=claro|escuro` declara os valores aceitos; o primeiro é o padrão. Outro valor é erro ([R418](/reference/errors/)).
- `.plano-$tom` escolhe o estilo pela variante, sem repetir propriedades.

### Componente, variável ou estilo?

| Quero… | Use |
| --- | --- |
| Um pacote de propriedades visuais | `@style` |
| A mesma estrutura com conteúdos diferentes | `@component` |
| Um valor compartilhado (tamanho, texto, opção) | `@let` |
| Várias estruturas geradas a partir de dados | `@data` + `::: for` + componente |
| Algo que muda enquanto o documento é mostrado | `@state` (veja [Interatividade](/guides/interactive/)) |

## Tokens no tema base

O tema base também aceita `@let` e `@data`. Um documento pode redefinir um `@let` do tema, do mesmo jeito que faz com `@color`.

## Erros comuns

| Código | Quando acontece |
| --- | --- |
| [R601](/reference/errors/) | `$nome` numa expressão, `if` ou `for` sem ter sido declarado. |
| [R602](/reference/errors/) | Tipos que não combinam, como `$texto * 2` ou um `if` sobre um número. |
| [R603](/reference/errors/) | Expressão mal escrita, como `$(1 +)`. |
| [R606](/reference/errors/) | `for` sobre algo que não é lista, ou repetições demais. |
| [R607](/reference/errors/) | `::: else` sem um `::: if` logo antes. |
| [R609](/reference/errors/) | `@let`, `@state` ou `@data` mal escrito. |
| [R610](/reference/errors/) | O mesmo nome declarado duas vezes. |
