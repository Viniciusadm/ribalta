---
title: Sintaxe
description: Referência rápida de toda a linguagem Ribalta, da estrutura do documento aos escapes.
---

Esta página resume a linguagem inteira em tabelas curtas. Para explicações passo a passo, veja os [guias](/docs/guides/writing/); para os valores aceitos por cada atributo, veja [Atributos](/docs/reference/attributes/).

## Estrutura do documento

Um documento tem duas partes, nesta ordem:

1. **Preâmbulo** (opcional): linhas que começam com `@`, só no topo do arquivo.
2. **Páginas**: todo o resto. O primeiro conteúdo que não é definição começa a página 1, e cada `---` começa uma página nova.

```ribalta
@title Resultados do trimestre
@theme paper

# Primeira página

--- {.dark align=center}
# Segunda página
```

### Definições do preâmbulo

| Definição | Forma | Efeito |
| --- | --- | --- |
| `@title` | `@title Texto` | Título do documento. |
| `@author` | `@author Nome` | Autor do documento. |
| `@theme` | `@theme paper` | Tema embutido: `paper` (padrão), `ink` ou `editorial`. |
| `@version` | `@version 2` | Versão da linguagem: `1` ou `2`. A versão atual é `2`. |
| `@page` | `@page 1920x1080`, `@page 800xauto`, `@page width=800 height=auto` | Tamanho das páginas, em pixels: largura `x` altura, de 16 a 16384; `auto` na altura faz a página crescer com o conteúdo. A forma com chaves só muda o que nomeia. Sem `@page`, vale o padrão do programa que compila o documento. Veja [Tamanhos e documentos](/docs/guides/pages/). |
| `@color` | `@color marca #ff6b4a` | Cria ou substitui um token de cor. |
| `@font` | `@font titulo "Bebas Neue" 400 700` | Cria ou substitui um token de fonte (família do Google Fonts e pesos de 100 a 900). |
| `@style` | `@style destaque {color=marca weight=bold}` | Cria um estilo reutilizável, aplicado com `.destaque`. |
| `@component` … `@end` | `@component nome {param param=padrão}` | Define um componente. O corpo vai até a linha `@end`. |
| `@let` | `@let espaco = lg` | Cria uma variável fixa, usada com `$espaco`. Veja [Variáveis](/docs/guides/data/). |
| `@data` … `@end` | `@data equipe` seguido de uma tabela | Cria uma lista de registros a partir da tabela. |
| `@state` | `@state detalhes = false` | Cria uma variável que muda enquanto o documento é mostrado. Veja [Interatividade](/docs/guides/interactive/). |
| `@transition` | `@transition fade` | Transição padrão ao entrar nas páginas: `fade`, `slide`, `zoom` ou `none`. |
| `@extern` | `@extern placar = 0` | Cria uma variável que quem mostra o documento define, como o placar de um jogo. Veja [Valores de fora](/docs/guides/interactive/#valores-de-fora-com-extern). |
| `@persist` | `@persist finais moedas` | Guarda esses `@state` no navegador, para o progresso continuar depois de fechar a página. Veja [Progresso salvo](/docs/guides/interactive/#progresso-salvo). |
| `@scorm` | `@scorm score=$pontos complete=#fim` | Nota e conclusão enviadas a um LMS no pacote SCORM. Veja [Nota e conclusão em LMS](/docs/guides/interactive/#nota-e-conclusão-em-lms-scorm). |
| `@mode` | `@mode game` | `default` (padrão) ou `game`: em `game`, as setas não saem das páginas e o programa que mostra o documento esconde o contador. Veja [Interatividade](/docs/guides/interactive/#páginas-sem-avanço-e-modo-jogo). |
| `@header` … `@end` | `@header {atributos}` | Cabeçalho repetido em todas as páginas, menos na capa. Veja [Cabeçalho, rodapé e camada livre](/docs/guides/header-footer/). |
| `@footer` … `@end` | `@footer {atributos}` | Rodapé repetido em todas as páginas, menos na capa. `@footer none` desliga o rodapé do tema do projeto. |
| `@overlay` … `@end` | `@overlay {atributos}` | Camada livre por cima do conteúdo, em todas as páginas menos na capa; blocos com `x`/`y` ficam relativos ao canto da página. `@overlay none` desliga a do tema do projeto. Veja [Camada livre](/docs/guides/header-footer/#camada-livre-overlay). |

Nomes de cores, fontes, estilos e componentes usam letras minúsculas, números e hífens, começando por uma letra. Linhas vazias e comentários podem aparecer entre as definições. Uma definição depois da primeira página é erro ([R103](/docs/reference/errors/)).

### Separador de páginas

| Forma | Efeito |
| --- | --- |
| `---` | Começa uma página nova. Três ou mais hífens, sozinhos na linha. |
| `--- {atributos}` | Começa uma página nova e aplica atributos à página inteira. |
| `--- {#id}` | Dá um id à página, usado em links internos como `[voltar](#id)` e em `click="go #id"`. |
| `--- {hidden}` | Tira a página do fluxo normal; ela só é alcançada por link ou `go`. |
| `--- {next=#id}` | Troca a página seguinte ao avançar. |
| `--- {width=1080 height=1920}` | Muda o tamanho só desta página, em pixels. `height=auto` faz a página crescer com o conteúdo. |

Logo depois do separador, linhas `@let` e `@state` criam variáveis que só existem naquela página.

- Se o documento começa com `---` logo depois do preâmbulo, esse separador configura a primeira página; não é criada uma página vazia.
- Um `---` dentro de um bloco de código não separa páginas.
- O título da página é o primeiro título `#` dela; sem título, vale "Página N". Sem `#id`, o id da página é `page-N`.

## Markdown suportado

| Construção | Sintaxe | Observações |
| --- | --- | --- |
| Títulos | `# Título` … `###### Título` | Seis níveis. É preciso um espaço depois dos `#`. |
| Parágrafo | Linhas de texto seguidas | Cada quebra de linha vira uma quebra de linha na página. Uma linha vazia separa parágrafos. |
| Negrito | `**texto**` | Só com asteriscos. |
| Itálico | `*texto*` | Só com asteriscos; `_texto_` não é itálico. |
| Riscado | `~~texto~~` | |
| Código em linha | `` `código` `` | O conteúdo não é interpretado. |
| Link | `[texto](https://…)` | Aceita `https://`, `http://`, `mailto:` e `#id-da-pagina`. |
| Trecho com atributos | `[texto]{color=accent}` | As chaves vêm coladas no `]`. |
| Imagem em linha | `![descrição](https://…)` | Dentro de um parágrafo; aceita atributos colados: `![x](…){h=60}`. |
| Imagem em bloco | `![descrição](https://…){fit=cover}` | Sozinha na linha. Aceita `https://`, `http://`, `data:image/…` e `asset:<id>`. |
| Lista | `- item`, `* item` ou `+ item` | Itens aninhados com indentação. Linhas indentadas continuam o item. |
| Lista numerada | `1. item` ou `1) item` | O primeiro número define o início da contagem. |
| Citação | `> texto` | Pode conter outros blocos, como listas e títulos. |
| Bloco de código | ` ```js ` … ` ``` ` | A linguagem é opcional; atributos podem vir no fim da linha de abertura. |
| Tabela | `\| A \| B \|` + `\| --- \| --- \|` | A segunda linha é obrigatória. `:---`, `:---:` e `---:` alinham a coluna. |

HTML não é interpretado: `<b>` aparece como texto na página.

## Atributos

Atributos ficam entre chaves e configuram o bloco, o trecho ou a página em que estão.

| Forma | Exemplo | Significado |
| --- | --- | --- |
| `.estilo` | `{.card}` | Aplica um estilo embutido, um `@style` ou um token de cor (`{.coral}` equivale a `{color=coral}`). |
| `#id` | `--- {#resumo}` | Id da página. Só tem efeito no separador. |
| `chave=valor` | `{size=5xl}` | Valor sem espaços. |
| `chave="valor"` | `{pad="sm lg"}` | Valor com espaços, entre aspas duplas ou simples. Use `\"` para uma aspa dentro do valor. |
| `chave` | `{italic}` | Liga um atributo booleano. Para desligar: `italic=false`. |
| `.estilo=condição` | `{.destaque=$ativo}` | Aplica o estilo só quando a condição é verdadeira. Com `hover`, enquanto o mouse está sobre o elemento. |

Valores booleanos aceitam `true`, `yes` e `sim` para ligar e `false`, `no` e `não` para desligar.

### Onde os atributos ficam

| Onde | Posição | Exemplo |
| --- | --- | --- |
| Títulos, parágrafos, itens de lista, linha de abertura de código | No fim da linha, **com espaço antes** | `# Título {size=5xl}` |
| Blocos `:::` e `::` | No fim da linha do bloco | `::: box {.card}` |
| Separador de página | Depois de `---` | `--- {bg=ink color=white}` |
| Trecho de texto | **Colado** no `]` | `[38%]{color=accent}` |
| Imagem | **Colado** no `)` | `![Foto](https://…){fit=cover}` |

Num parágrafo de várias linhas, os atributos no fim de uma linha valem para o parágrafo inteiro e encerram o parágrafo. Estilos (`.estilo`) são aplicados primeiro, e os pares `chave=valor` escritos no próprio bloco prevalecem sobre eles.

## Blocos

| Forma | Uso |
| --- | --- |
| `::: nome {atributos}` … `:::` | Bloco contêiner. Um `:::` sozinho na linha fecha o último bloco aberto. |
| `:: nome valor {atributos}` | Bloco de uma linha só, sem conteúdo. |

Blocos podem ser aninhados. A indentação é opcional e serve apenas para leitura.

```ribalta
::: columns {widths="2 1"}
::: stack
## Esquerda
Texto
:::
:: icon rocket {size=3xl color=accent}
:::
```

| Bloco | Forma | Função |
| --- | --- | --- |
| `columns` | `:::` | Colunas lado a lado; cada filho é uma coluna. |
| `stack` | `:::` | Empilha os filhos na vertical. |
| `row` | `:::` | Coloca os filhos em linha, na horizontal. |
| `grid` | `:::` | Grade com número fixo de colunas. |
| `box` | `:::` | Caixa com fundo, borda e espaçamento. |
| `steps` | `:::` | Revela cada filho, ou cada item de lista, um por vez. |
| `notes` | `:::` | Notas para quem mostra o documento; nunca são desenhadas na página. |
| `chart` | `:::` ou `::` | Gráfico a partir de uma tabela ou de `data=$lista`. |
| `table` | `::` | Tabela a partir de `data=$lista`. |
| `if` / `else` | `:::` | Mostra o conteúdo conforme uma condição. |
| `for` | `:::` | Repete o conteúdo para cada item de uma lista. |
| `slot` | `:::` | Preenche um espaço nomeado de um componente. |
| `icon` | `::` | Ícone: `:: icon rocket`. |
| `spacer` | `::` | Espaço vazio: `:: spacer lg`. |
| `divider` | `::` | Linha divisória. |
| `input` | `::` | Campo de texto ligado a um `@state`: `:: input {bind=$nome}`. |
| `sound` | `::` | Som tocado por `play` e `stop`: `:: sound https://… {name=porta}`. |

Detalhes de cada bloco em [Blocos](/docs/reference/blocks/).

## Componentes

| Forma | Significado |
| --- | --- |
| `@component nome {a b=padrão}` | Define o componente `nome` com o parâmetro obrigatório `a` e o parâmetro `b` com valor padrão. |
| `@end` | Fecha a definição. |
| `:: nome {a=valor}` | Usa o componente sem conteúdo. |
| `::: nome {a=valor}` … `:::` | Usa o componente com conteúdo, que vai para `$content`. |
| `::: slot lado` … `:::` | Dentro do uso de um componente, preenche o espaço nomeado `$lado`. |
| `{step=N}` no uso | Revela o componente inteiro na etapa `N`. |
| `{if=… click=…}` no uso | Mostra o componente conforme a condição e define ações de clique. |
| `{tipo=a\|b\|c}` na definição | Parâmetro com valores aceitos; o primeiro é o padrão. |

Componentes recebem apenas parâmetros nomeados: `.estilo`, `#id` e texto solto depois do nome são erro. Componentes não podem usar a si mesmos, direta ou indiretamente. Um parâmetro pode receber um registro ou uma lista inteira: `:: cartao {p=$pessoa}`.

## Marcadores `$`

| Marcador | Onde | Efeito |
| --- | --- | --- |
| `$param` | No corpo de um componente, em texto ou valor de atributo | Trocado pelo valor do parâmetro: `## $titulo`, `{color=$cor}`. |
| `$content` | Sozinho numa linha do corpo | Recebe o conteúdo passado entre `::: nome` e `:::`. |
| `$nome` | Sozinho numa linha do corpo, quando `nome` não é parâmetro | Espaço nomeado, preenchido com `::: slot nome`. Fica vazio se não for preenchido. |
| `$page`, `$pages`, `$title`, `$author`, `$date` | Dentro de `@header`, `@footer` e `@overlay` | Número desta página, total de páginas, título, autor e data de hoje. Outro nome é erro ([R522](/docs/reference/errors/)). |
| `$variavel`, `$item.campo` | Em qualquer lugar | Valor de um `@let`, `@data`, `@state`, variável do `for` ou campo de registro. |
| `$( expressão )` | Em qualquer lugar | Resultado de uma expressão, como `$($preco * 12)`. |
| `$step` | Em qualquer lugar | Etapa atual da página. |
| `$elapsed` | Em qualquer lugar | Segundos desde que a página abriu. Veja [Tempo](/docs/guides/interactive/#tempo). |

Um `$palavra` que não é variável declarada é texto comum, então `US$ 10` continua igual. A exceção é `$content` sozinho numa linha fora de um componente, que é erro ([R401](/docs/reference/errors/)). Para um `$` literal antes de um nome declarado, escreva `\$nome`.

## Expressões

Expressões aparecem em `$( … )`, em `@let`, nas condições de `if` e `.estilo=`, nas listas de `for` e nas ações de `click`.

| Elemento | Exemplos |
| --- | --- |
| Valores | `42`, `3.5`, `"com espaços"`, `palavra`, `true`, `false`, `[a, b, c]`, `1..5` |
| Variáveis | `$total`, `$pessoa.nome`, `$vendas.receita` (coluna inteira) |
| Operadores | `+ - * / %`, `== != < <= > >=`, `in`, `and or not`, `condição ? a : b` |
| Funções | `len`, `min`, `max`, `sum`, `round`; em ações, `random` e `pick` |

Uma palavra sem `$` é texto. Um hífen seguido de letra faz parte do nome (`$titulo-curto`); para subtrair uma variável de outra, use espaços: `$a - $b`.

## Ações de clique

| Ação | Efeito |
| --- | --- |
| `$x = expressão` | Muda um `@state`. |
| `toggle $x` | Inverte um `@state` verdadeiro/falso. |
| `add $x valor`, `remove $x valor` | Põe ou tira um texto de uma lista do `@state`. |
| `clear $x` | Esvazia uma lista do `@state`. |
| `play nome`, `stop nome` | Toca ou para um `:: sound` da página. |
| `go #id` | Vai para a página `id`. |
| `go back` | Volta para a posição anterior ao último salto. |
| `go next`, `go prev` | Avança ou volta uma etapa. |

Várias ações são separadas por `;`: `click="$plano = pro; go #detalhe"`.

## Comentários

Uma linha que começa com `//` (com ou sem espaços antes) é ignorada, tanto no preâmbulo quanto nas páginas.

```ribalta
// Revisar estes números antes da reunião
# Resultados
```

- `//` no meio de uma linha é texto comum.
- Dentro de blocos de código e de `::: notes`, linhas com `//` são mantidas.

## Escapes

| Escreva | Para obter |
| --- | --- |
| `\*`, `\~`, `` \` ``, `\[`, `\]`, `\!` | O caractere literal, sem formatação. |
| `\{` | Uma chave literal; impede que `{…}` no fim da linha vire atributos. |
| `\\|` | Uma barra vertical dentro de uma célula de tabela. |
| `\#` no início da linha | Um `#` literal, sem virar título. |
| `\"` dentro de um valor entre aspas | Uma aspa dentro do valor. |

A barra invertida escapa qualquer sinal de pontuação ASCII. Antes de outros caracteres, ela aparece normalmente.

## Exemplo completo

```ribalta preview
// Preâmbulo: só definições, no topo do arquivo
@title Relatório trimestral
@theme paper
@color marca #2f6fdb
@style destaque {color=marca weight=bold}
@component metrica {valor rotulo cor=accent}
::: box {.card align=center}
$valor {size=4xl weight=black color=$cor}
$rotulo {.muted}
:::
@end

// Página 1: o primeiro conteúdo começa a página
Relatório trimestral {.kicker}
# Crescemos [38%]{.destaque} em receita {size=5xl}
Novos clientes, mais retenção
e uma equipe maior. {.lead}

// Página 2: separador com id e atributos de página
--- {#numeros align=center}
## Números que importam
::: grid {cols=3 gap=lg}
:: metrica {valor=+38% rotulo=Receita}
:: metrica {valor=12k rotulo=Clientes cor=green}
:: metrica {valor=4,8 rotulo=Avaliação cor=blue}
:::

::: notes
Destacar a avaliação dos clientes.
:::

// Página 3: fundo em gradiente e revelação por etapas
--- {bg=ink..violet color=white}
## Próximos passos
::: steps
- Abrir duas novas regiões
- Lançar o plano anual
- Contratar o time de suporte
:::
Detalhes na [página de números](#numeros). {opacity=0.7}
```
