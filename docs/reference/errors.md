---
title: Mensagens de erro
description: Todos os códigos de diagnóstico do compilador, com causa, exemplo e correção.
---

O compilador da Ribalta nunca para no primeiro problema: ele sempre gera o melhor documento possível e uma lista de diagnósticos. Cada diagnóstico tem:

| Parte | Exemplo |
| --- | --- |
| Gravidade | **Erro** (algo foi descartado ou não funciona) ou **aviso** (funciona, mas provavelmente não como você queria). |
| Código | `R209`. O código é estável e não muda entre versões. |
| Posição | Linha e coluna do trecho com problema. |
| Mensagem | atributo desconhecido: `colr` |
| Dica | Você quis dizer `color`? |

Erros dentro de um componente são indicados na linha onde o componente é **usado**, com o sufixo "(no componente `nome`)". Erros em um `@style` aparecem uma vez, na definição, e não em cada uso.

| Família | Assunto |
| --- | --- |
| [F1xx](#f1xx-estrutura) | Estrutura: blocos, tabelas, código, gráficos e ícones. |
| [F2xx](#f2xx-atributos) | Atributos: sintaxe das chaves, estilos e nomes de atributos. |
| [F3xx](#f3xx-valores) | Valores de atributos. |
| [F4xx](#f4xx-componentes) | Componentes e espaços nomeados. |
| [F5xx](#f5xx-definições-e-tema-do-projeto) | Definições do preâmbulo e tema do projeto. |
| [F6xx](#f6xx-variáveis-expressões-e-estado) | Variáveis, dados, expressões, repetição e estado. |
| [F7xx](#f7xx-navegação) | Navegação entre páginas. |

## F1xx: estrutura

### R101 · Fechamento sem bloco aberto

**Erro.** Mensagem: `:::` fecha um bloco, mas nenhum bloco está aberto aqui.

Um `:::` sozinho na linha fecha o último bloco aberto. Aqui não há nenhum bloco aberto, geralmente porque há um `:::` a mais.

```ribalta
# Resultados
Crescemos 38%.
:::
```

**Como corrigir:** remova a linha ou abra um bloco antes dela, como `::: columns`.

### R102 · Bloco não fechado

**Erro.** Mensagem: o bloco `columns` foi aberto mas nunca fechado.

Todo bloco aberto com `::: nome` precisa de uma linha `:::` que o feche. Cada página é lida separadamente, então o bloco precisa ser fechado antes do próximo `---`.

```ribalta
::: columns
Esquerda
Direita
```

**Como corrigir:** adicione uma linha contendo apenas `:::` no fim do bloco.

### R103 · Definição depois da primeira página

**Erro.** Mensagem: `@theme` só pode ser usado no início do documento, antes da primeira página.

Definições com `@` (`@title`, `@theme`, `@color`, `@style`, `@component` e as demais) só valem no preâmbulo. Depois que a primeira página começa, elas são descartadas.

```ribalta
# Abertura

---
@theme ink
# Segunda página
```

**Como corrigir:** mova a definição para o topo do arquivo, antes de qualquer conteúdo.

### R104 · Bloco sem nome

**Erro.** Mensagem: falta o nome do bloco depois de `:::`.

Uma linha que começa com `:::` ou `::` e tem outro conteúdo precisa ter o nome do bloco logo em seguida.

```ribalta
::: {.card}
Texto
:::
```

**Como corrigir:** escreva o nome do bloco antes dos atributos, como `::: box {.card}`.

### R105 · Bloco de código sem fechamento

**Erro.** Mensagem: bloco de código sem fechamento.

Um bloco de código aberto com ` ``` ` precisa de outra linha com ` ``` ` para terminar. Sem ela, todo o resto da página vira código.

````ribalta
```js
console.log("oi")
````

**Como corrigir:** feche o bloco com uma linha contendo ` ``` `.

### R106 · Tabela sem linha separadora

**Erro.** Mensagem: tabela sem linha separadora depois do cabeçalho.

A segunda linha de uma tabela precisa ser a linha separadora, que divide o cabeçalho dos dados.

```ribalta
| Produto | Preço |
| Café | 12 |
```

**Como corrigir:** adicione uma linha como `| --- | --- |` logo abaixo do cabeçalho.

### R107 · Linha de tabela com outro número de colunas

**Aviso.** Mensagem: esta linha tem 3 colunas, mas o cabeçalho tem 2.

Uma linha da tabela tem mais ou menos células que o cabeçalho.

```ribalta
| Produto | Preço |
| --- | --- |
| Café | 12 | extra |
```

**Como corrigir:** deixe todas as linhas com o mesmo número de células do cabeçalho. Para escrever `|` dentro de uma célula, use `\|`.

### R108 · Bloco desconhecido

**Erro.** Mensagem: bloco desconhecido: `colums`.

O nome depois de `:::` ou `::` não é um bloco embutido nem um componente definido no documento ou no tema do projeto. A dica sugere o nome mais parecido.

```ribalta
::: colums
Texto
:::
```

**Como corrigir:** corrija o nome (aqui, `columns`) ou defina o componente com `@component`. Veja a lista em [Blocos](/docs/reference/blocks/).

### R109 · Bloco de uma linha com conteúdo

**Aviso.** Mensagem: `divider` não aceita conteúdo; use `:: divider` em uma linha só.

`icon`, `spacer` e `divider` não têm conteúdo. Abertos com `:::`, o conteúdo entre eles e o `:::` de fechamento é ignorado.

```ribalta
::: divider
Texto
:::
```

**Como corrigir:** use a forma de uma linha, `:: divider`, e coloque o texto fora do bloco.

### R110 · Texto ignorado depois do nome do bloco

**Aviso.** Mensagem: o texto `3` depois de `grid` será ignorado.

Só `icon` e `spacer` recebem um valor depois do nome. Nos outros blocos embutidos, o texto solto é ignorado.

```ribalta
::: grid 3
A
B
C
:::
```

**Como corrigir:** use atributos entre chaves, como `::: grid {cols=3}`.

### R111 · Gráfico sem tabela

**Erro.** Mensagem: o gráfico precisa de uma tabela com os dados.

O bloco `chart` desenha os dados de uma tabela Markdown que fica dentro dele. Sem tabela, o gráfico não é desenhado.

```ribalta
::: chart {type=bar}
Vendas por mês
:::
```

**Como corrigir:** coloque uma tabela dentro do bloco. A primeira coluna são os rótulos e as demais são as séries. Veja [Gráficos](/docs/guides/charts/).

### R112 · Valor do gráfico não é número

**Erro.** Mensagem: `doze` não é um número.

Todas as células de dados de um gráfico, fora a primeira coluna, precisam ser números. A célula inválida conta como 0.

```ribalta
::: chart
| Mês | Vendas |
| --- | ---: |
| Jan | doze |
:::
```

**Como corrigir:** use números como `42`, `3.5` ou `3,5`. Um `%` no fim é aceito.

### R113 · Gráfico sem séries

**Erro.** Mensagem: a tabela do gráfico precisa de pelo menos uma coluna de valores.

A tabela tem só a coluna de rótulos, então não há o que desenhar.

```ribalta
::: chart
| Mês |
| --- |
| Jan |
:::
```

**Como corrigir:** adicione pelo menos uma coluna de valores à tabela.

### R114 · Ícone sem nome

**Erro.** Mensagem: informe o nome do ícone.

```ribalta
:: icon {size=3xl}
```

**Como corrigir:** escreva o nome depois de `icon`, como `:: icon rocket {size=3xl}`. Os nomes estão em [Ícones](/docs/reference/icons/).

### R115 · Ícone desconhecido

**Erro.** Mensagem: ícone desconhecido: `foguete`.

O nome não está na lista de ícones embutidos. Quando há um nome parecido, a dica o sugere.

```ribalta
:: icon foguete
```

**Como corrigir:** use um nome da lista em [Ícones](/docs/reference/icons/), como `rocket`.

### R116 · Campo sem variável

**Erro.** Mensagem: `input` precisa de `bind` com a variável que guarda o texto.

```ribalta
@state nome = ""

# Quem é você?
:: input {placeholder="Seu nome"}
```

**Como corrigir:** ligue o campo a um `@state` de texto ou número com `bind`: `:: input {bind=$nome placeholder="Seu nome"}`. O mesmo erro aparece quando `bind` não começa com `$`.

### R117 · Som sem endereço

**Erro.** Mensagem: `sound` precisa do endereço do arquivo de áudio.

```ribalta
# A porta
:: sound {name=porta}
```

**Como corrigir:** escreva o endereço depois de `sound`: `:: sound https://exemplo.com/porta.mp3 {name=porta}`.

## F2xx: atributos

### R201 · Ponto ou cerquilha sem nome

**Erro.** Mensagem: `.` precisa ser seguido de um nome.

Dentro das chaves, `.` precisa vir colado ao nome de um estilo e `#` ao nome de um id.

```ribalta
Texto {. color=accent}
```

**Como corrigir:** escreva o nome junto, como `{.destaque}` ou `{#abertura}`, ou remova o caractere solto.

### R202 · Aspas não fechadas

**Erro.** Mensagem: aspas não fechadas no valor do atributo.

Um valor que começa com aspas precisa terminar com a mesma aspa.

```ribalta
![Foto](https://example.com/foto.jpg){fit="cover}
```

**Como corrigir:** feche o valor com a mesma aspa que o abriu: `{fit="cover"}`. No fim de títulos, parágrafos e blocos, chaves com aspas abertas não são reconhecidas como atributos e aparecem como texto na página.

### R203 · Atributo sem valor

**Erro.** Mensagem: o atributo `color` está sem valor depois de `=`.

Há um `=` sem nada depois. Normalmente vem acompanhado de um [R301](#r301--valor-inválido), porque o valor vazio também é inválido.

```ribalta
# Título {color=}
```

**Como corrigir:** informe o valor (`{color=accent}`) ou remova o atributo.

### R204 · Trecho inválido em atributos

**Erro.** Mensagem: trecho inválido em atributos: `@destaque`.

Dentro das chaves só são aceitos `.estilo`, `#id`, `chave=valor`, `chave="valor com espaços"` e `chave`.

```ribalta
Texto {@destaque}
```

**Como corrigir:** use a forma certa, aqui `{.destaque}`. Nomes de atributos começam com uma letra.

### R205 · Id fora do separador de página

**Aviso.** Mensagem: `#id` só tem efeito no separador de página (--- {#id}).

Ids identificam páginas, para links como `[voltar](#abertura)`. Em outros blocos, o id é ignorado.

```ribalta
# Abertura {#inicio}
```

**Como corrigir:** coloque o id no separador da página, como `--- {#inicio}`.

### R206 · Estilo que referencia a si mesmo

**Erro.** Mensagem: o estilo `.laco` referencia a si mesmo.

Um `@style` inclui a si mesmo, diretamente ou por meio de outros estilos. Como o estilo ainda não existe enquanto a própria definição é lida, o compilador também mostra um [R207](#r207--estilo-desconhecido) no mesmo lugar.

```ribalta
@style laco {.laco color=accent}

Texto {.laco}
```

**Como corrigir:** remova a referência circular. Um estilo só pode incluir estilos definidos antes dele.

### R207 · Estilo desconhecido

**Erro.** Mensagem: estilo desconhecido: `.destaqe`.

O nome depois do `.` não é um estilo embutido, um `@style` nem um token de cor. A dica sugere o nome mais parecido.

```ribalta
Texto {.destaqe}
```

**Como corrigir:** corrija o nome ou declare o estilo no início do documento com `@style nome {atributos}`. Veja os [estilos embutidos](/docs/reference/themes/#estilos-embutidos).

### R208 · Id de página repetido

**Aviso.** Mensagem: já existe outra página com o id `#resumo`.

Duas páginas têm o mesmo id, então um link `#resumo` só pode levar a um deles.

```ribalta
--- {#resumo}
# Resumo

--- {#resumo}
# Conclusão
```

**Como corrigir:** dê um id diferente a cada página.

### R209 · Atributo desconhecido

**Erro.** Mensagem: atributo desconhecido: `colr`.

O nome não é um atributo de estilo nem um atributo próprio do bloco em que aparece. A dica sugere o nome mais parecido ou lista os atributos aceitos ali.

```ribalta
# Título {colr=coral}
```

**Como corrigir:** corrija o nome. Atributos próprios, como `cols` ou `fit`, só valem nos blocos certos. Veja [Atributos](/docs/reference/attributes/).

## F3xx: valores

### R210 · Tecla sem clique

**Aviso.** Mensagem: `key` não faz nada sem `click`.

A tecla aciona as ações de `click` do elemento; sem elas, apertar a tecla não muda nada.

```ribalta
# Mapa
[Norte]{key=up}
```

**Como corrigir:** diga o que a tecla faz com `click`: `[Norte]{click="$y = $y - 1" key=up}`.

### R301 · Valor inválido

**Erro.** Mensagem: depende do atributo, por exemplo tamanho inválido: `gigante` ou cor desconhecida: `corall`.

O atributo existe, mas o valor não é aceito. O atributo é ignorado, e a dica diz quais valores são válidos.

```ribalta
# Título {size=gigante color=corall}
```

Situações comuns:

| Situação | Exemplo inválido | Exemplo válido |
| --- | --- | --- |
| Cor desconhecida ou hexadecimal malformado | `color=corall`, `color=#12` | `color=coral`, `color=#123` |
| Opacidade de cor fora de 0 a 100 | `color=accent/150` | `color=accent/40` |
| Nome fora da escala ou número fora da faixa | `size=gigante`, `pad=900` | `size=5xl`, `pad=xl` |
| Fonte não declarada | `font=titulo` | `font=display` ou `@font titulo …` |
| Peso inválido | `weight=450` | `weight=500` |
| Booleano inválido | `italic=talvez` | `italic` ou `italic=false` |
| Alinhamento inválido | `align=middle` | `align=center` |
| Sombra inválida | `shadow=xl` | `shadow=lg` |
| Borda com espessura acima de 40 | `border="80 coral"` | `border="4 coral"` |
| Medida inválida | `w=200%`, `w=metade` | `w=50%`, `w=480` |
| Atributo que precisa de valor, usado sozinho | `{size}` | `{size=xl}` |
| Endereço não permitido em link ou imagem | `[x](javascript:…)`, `![x](arquivo.png)` | `https://…`, `asset:<id>` |
| Ajuste de imagem inválido | `fit=stretch` | `fit=cover` |
| Colunas da grade fora de 1 a 12 | `cols=20` | `cols=4` |
| Proporções de colunas inválidas | `widths="a b"` | `widths="2 1"` |
| Tipo de gráfico inválido | `type=radar` | `type=line` |
| Tamanho de `spacer` inválido | `:: spacer enorme` | `:: spacer 3xl` |
| Etapa fora de 1 a 99 | `step=0` | `step=1` |

**Como corrigir:** use um dos valores indicados na dica. As escalas e faixas completas estão em [Atributos](/docs/reference/attributes/).

### R302 · `widths` não bate com as colunas

**Aviso.** Mensagem: `widths` tem 3 valores, mas há 1 colunas.

O número de proporções em `widths` é diferente do número de filhos do `columns`.

```ribalta
::: columns {widths="2 1 1"}
Esquerda
:::
```

**Como corrigir:** informe uma proporção para cada coluna, ou ajuste as colunas. Lembre que cada filho direto é uma coluna; agrupe vários elementos com `::: stack`.

## F4xx: componentes

### R401 · `$content` fora de componente

**Erro.** Mensagem: `$content` só pode ser usado dentro de um @component.

`$content` marca onde entra o conteúdo passado a um componente, então só faz sentido no corpo de um `@component`.

```ribalta
# Título
$content
```

**Como corrigir:** remova a linha ou mova-a para o corpo de um componente. Para escrever o texto literal, use `\$content`.

### R404 · Componente que usa a si mesmo

**Erro.** Mensagem: o componente `eco` usa a si mesmo.

Um componente usa a si mesmo, diretamente ou por meio de outros componentes. Componentes não têm recursão, então o uso é descartado.

```ribalta
@component eco
:: eco
@end

:: eco
```

**Como corrigir:** remova o uso circular do corpo do componente.

### R405 · Componentes aninhados demais

**Erro.** Mensagem: componentes aninhados demais (limite: 16 níveis).

Um componente usa outro, que usa outro, e assim por diante, por mais de 16 níveis. O exemplo abaixo está abreviado: com as 17 definições completas, o erro aparece no uso de `n1`.

```ribalta
@component n1
:: n2
@end
@component n2
:: n3
@end
// n3 a n16 seguem o mesmo padrão, cada um usando o seguinte
@component n17
Fim
@end

:: n1
```

**Como corrigir:** simplifique a cadeia de componentes, juntando níveis intermediários.

### R406 · Estilo ou id no uso de um componente

**Erro.** Mensagem: componentes recebem apenas parâmetros, não .estilos nem #id.

Ao usar um componente, as chaves só aceitam parâmetros nomeados.

```ribalta
@component selo {texto}
::: box {.card}
$texto
:::
@end

:: selo {.dark texto=Novo}
```

**Como corrigir:** envolva o componente num bloco com o estilo (`::: box {.dark}` … `:: selo {texto=Novo}` … `:::`) ou crie um parâmetro para isso.

### R407 · Parâmetro inexistente

**Erro.** Mensagem: o componente `selo` não tem o parâmetro `cor`.

O parâmetro não foi declarado na definição do componente. A dica lista os parâmetros existentes.

```ribalta
@component selo {texto}
::: box {.card}
$texto
:::
@end

:: selo {texto=Novo cor=accent}
```

**Como corrigir:** corrija o nome ou declare o parâmetro, como `@component selo {texto cor=accent}`.

### R408 · Texto solto depois do nome do componente

**Erro.** Mensagem: componentes recebem parâmetros nomeados entre chaves.

Componentes não recebem valores posicionais: todo parâmetro precisa de nome.

```ribalta
@component selo {texto=Novo}
::: box {.card}
$texto
:::
@end

:: selo Promoção
```

**Como corrigir:** passe o valor como parâmetro nomeado: `:: selo {texto=Promoção}`.

### R409 · Parâmetro obrigatório ausente

**Erro.** Mensagem: falta o parâmetro obrigatório `texto` do componente `selo`.

Parâmetros declarados sem valor padrão são obrigatórios.

```ribalta
@component selo {texto}
::: box {.card}
$texto
:::
@end

:: selo
```

**Como corrigir:** informe o parâmetro (`:: selo {texto=Novo}`) ou dê a ele um valor padrão na definição (`@component selo {texto=Novo}`).

### R410 · Marcador `$` não declarado

**Erro.** Mensagem: o componente `selo` usa `$titulo`, que não é um parâmetro declarado.

O corpo do componente usa `$titulo` no meio de um texto ou atributo, mas `titulo` não está na lista de parâmetros.

```ribalta
@component selo {texto}
::: box {.card}
## $titulo
$texto
:::
@end

:: selo {texto=Novo}
```

**Como corrigir:** declare o parâmetro (`@component selo {texto titulo}`) ou corrija o nome. Sozinho numa linha, `$nome` é um espaço nomeado e não gera este erro.

Quando o nome existe só onde o componente é usado (um `@state` ou `@let` da página, ou a variável de um `::: for`), a mensagem diz que ele só existe ali: declare o parâmetro e passe o valor, como em `:: selo {texto=Novo titulo=$titulo}`. Os `@state` e `@let` do início do documento não precisam disso.

### R411 · Conteúdo ignorado pelo componente

**Aviso.** Mensagem: o componente `selo` não usa `$content`; o conteúdo será ignorado.

O componente foi usado com `:::` e recebeu conteúdo, mas o corpo não tem `$content`.

```ribalta
@component selo {texto}
::: box {.card}
$texto
:::
@end

::: selo {texto=Novo}
Este parágrafo é ignorado.
:::
```

**Como corrigir:** adicione `$content` ao corpo do componente ou use a forma de uma linha (`:: selo {texto=Novo}`).

### R412 · Nome de bloco embutido

**Erro.** Mensagem: `box` já é um bloco da linguagem e não pode ser redefinido.

Os nomes `stack`, `row`, `columns`, `grid`, `box`, `steps`, `notes`, `chart`, `icon`, `spacer`, `divider` e `slot` são reservados.

```ribalta
@component box
Texto
@end
```

**Como corrigir:** escolha outro nome, como `caixa` ou `cartao`.

### R413 · Componente definido duas vezes

**Erro.** Mensagem: o componente `selo` foi definido mais de uma vez.

O mesmo documento define dois componentes com o mesmo nome. A segunda definição é descartada.

```ribalta
@component selo
Novo
@end
@component selo
Promoção
@end
```

**Como corrigir:** remova uma das definições ou renomeie uma delas. Redefinir no documento um componente do tema do projeto é permitido e não gera este erro.

### R414 · Espaço nomeado inexistente

**Erro.** Mensagem: o componente `dupla` não tem o espaço `lado-c`.

`::: slot` preenche um espaço que o componente não declara. A dica sugere o nome mais parecido.

```ribalta
@component dupla {titulo}
## $titulo
$content
$lado-b
@end

::: dupla {titulo=Comparação}
Antes
::: slot lado-c
Depois
:::
:::
```

**Como corrigir:** use o nome de um espaço existente (aqui, `lado-b`) ou declare o espaço no componente com `$lado-c` sozinho numa linha.

### R415 · `::: slot` sem nome

**Erro.** Mensagem: informe o nome do espaço.

```ribalta
@component dupla {titulo}
## $titulo
$content
$lado-b
@end

::: dupla {titulo=Comparação}
Antes
::: slot
Depois
:::
:::
```

**Como corrigir:** escreva o nome do espaço depois de `slot`, como `::: slot lado-b`.

### R416 · `::: slot` fora de componente

**Erro.** Mensagem: `::: slot` só pode ser usado dentro de um componente.

`::: slot` só funciona como filho direto do uso de um componente contêiner. Fora dele, o compilador também avisa que o nome é ignorado ([R110](#r110--texto-ignorado-depois-do-nome-do-bloco)).

```ribalta
::: slot lado-b
Depois
:::
```

**Como corrigir:** coloque o `::: slot` dentro de `::: componente` … `:::`, ou remova o bloco.

### R417 · Espaço preenchido duas vezes

**Erro.** Mensagem: o espaço `lado-b` foi preenchido mais de uma vez.

```ribalta
@component dupla {titulo}
## $titulo
$content
$lado-b
@end

::: dupla {titulo=Comparação}
::: slot lado-b
Depois
:::
::: slot lado-b
De novo
:::
:::
```

**Como corrigir:** junte o conteúdo num único `::: slot lado-b`.

### R418 · Valor fora da lista do parâmetro

**Erro.** Mensagem: valor inválido para `tipo`: `primari`.

O parâmetro declara os valores aceitos com `|`, e o valor usado não está entre eles. O componente usa o primeiro valor da lista no lugar.

```ribalta
@component botao {texto tipo=primary|secondary}
**$texto**
@end

:: botao {texto=Enviar tipo=primari}
```

**Como corrigir:** use um dos valores declarados, como `tipo=primary`.

### R419 · Parâmetro com nome reservado

**Erro.** Mensagem: `key` não pode ser parâmetro: ele já é um atributo de quem usa o componente.

`step`, `if`, `click` e `key` são atributos que qualquer uso de componente aceita. Um parâmetro com o mesmo nome os esconderia: `:: botao {key=1}` preencheria o parâmetro, e o atalho de teclado sumiria.

```ribalta
@component botao {key texto}
**$key** $texto
@end

# Menu
:: botao {key=1 texto=Começar}
```

**Como corrigir:** dê outro nome ao parâmetro, como `tecla`: `@component botao {tecla texto}`. No uso, `key=1` volta a ser o atalho.

## F5xx: definições e tema do projeto

### R501 · Definição sem valor

**Erro.** Mensagem: `@title` precisa de um valor.

`@title`, `@author`, `@theme`, `@version` e `@page` precisam de um valor na mesma linha.

```ribalta
@title

# Abertura
```

**Como corrigir:** informe o valor, como `@title Relatório anual`, ou remova a linha.

### R502 · `@color` incompleto

**Erro.** Mensagem: `@color` espera um nome e um valor.

```ribalta
@color

# Abertura
```

**Como corrigir:** informe o nome do token e a cor, como `@color marca #ff6b4a`.

### R503 · `@style` incompleto

**Erro.** Mensagem: `@style` espera um nome e atributos.

```ribalta
@style destaque

# Abertura
```

**Como corrigir:** informe o nome e os atributos entre chaves, como `@style destaque {color=coral weight=bold}`.

### R504 · Bloco sem `@end`

**Erro.** Mensagem: `@component` sem `@end` (ou `@header`, `@footer`, `@overlay`).

Sem `@end`, todo o restante do documento é lido como corpo do bloco, e o documento fica sem páginas.

```ribalta
@component selo {texto}
::: box {.card}
$texto
:::

# Abertura
```

**Como corrigir:** termine o `@component`, `@header`, `@footer` ou `@overlay` com uma linha `@end`.

### R505 · `@end` sem bloco aberto

**Erro.** Mensagem: `@end` sem um bloco aberto.

```ribalta
@end

# Abertura
```

**Como corrigir:** remova o `@end` ou adicione antes dele o `@component`, `@header`, `@footer` ou `@overlay` que ele deveria fechar.

### R506 · Definição desconhecida

**Erro.** Mensagem: definição desconhecida: `@tema`.

O nome depois de `@` não é uma definição da linguagem. A dica sugere a mais parecida.

```ribalta
@tema ink

# Abertura
```

**Como corrigir:** use uma das definições: `@title`, `@author`, `@theme`, `@version`, `@page`, `@color`, `@font`, `@style`, `@component`, `@header`, `@footer`, `@overlay` ou `@end`.

### R507 · Nome de fonte sem aspas de fechamento

**Erro.** Mensagem: nome da fonte sem aspas de fechamento.

```ribalta
@font titulo "Bebas Neue 400

# Abertura
```

**Como corrigir:** feche as aspas depois do nome da família: `@font titulo "Bebas Neue" 400`.

### R508 · Família de fonte inválida

**Erro.** Mensagem: nome de fonte inválido: `Open-Sans`.

O nome da família só pode ter letras, números e espaços, e não pode ser vazio.

```ribalta
@font titulo "Open-Sans"

# Abertura
```

**Como corrigir:** use o nome da família como aparece no Google Fonts, entre aspas: `@font titulo "Open Sans"`.

### R509 · Peso de fonte inválido

**Erro.** Mensagem: peso de fonte inválido: `450`.

```ribalta
@font titulo "Inter" 450

# Abertura
```

**Como corrigir:** use pesos de 100 a 900, de 100 em 100, como `400` ou `500`.

### R510 · Componente sem nome

**Erro.** Mensagem: `@component` precisa de um nome.

```ribalta
@component {texto}
$texto
@end
```

**Como corrigir:** escreva o nome logo depois de `@component`: `@component selo {texto}`.

### R511 · Nome de componente inválido

**Erro.** Mensagem: nome de componente inválido: `Selo`.

Nomes de componente começam com letra minúscula e usam apenas letras minúsculas, números e hífens.

```ribalta
@component Selo {texto}
$texto
@end
```

**Como corrigir:** use um nome como `selo` ou `selo-novo`.

### R512 · Parâmetro de componente com estilo ou id

**Erro.** Mensagem: parâmetros de componente não aceitam .estilo nem #id.

A lista entre chaves de `@component` declara parâmetros, não estilos.

```ribalta
@component selo {.card texto}
$texto
@end
```

**Como corrigir:** remova o `.estilo` ou `#id` da lista e aplique o estilo num bloco do corpo, como `::: box {.card}`.

### R513 · Tema desconhecido

**Erro.** Mensagem: tema desconhecido: `dark`.

O tema não existe, e o documento usa o tema `paper`.

```ribalta
@theme dark

# Abertura
```

**Como corrigir:** use `paper`, `ink` ou `editorial`. Veja [Temas embutidos](/docs/reference/themes/).

`@theme` não cria nem dá nome a um tema: ele só escolhe a base. Para ter um tema seu, mantenha uma base e ajuste com `@color`, `@font` e `@style`. Veja [Temas](/docs/guides/themes/).

### R514 · Versão da linguagem não suportada

**Erro.** Mensagem: versão da linguagem não suportada: `3`.

```ribalta
@version 3

# Abertura
```

**Como corrigir:** use `@version 2`, a versão atual, ou remova a linha.

### R515 · Tamanho de página inválido

**Erro.** Mensagem: tamanho de página inválido: `16:9`.

`@page` recebe a largura e a altura da página em pixels, separadas por `x`, ou nas chaves `width` e `height`. Cada medida vai de 16 a 16384, e a altura também aceita `auto`. Uma proporção como `16:9` não é um tamanho.

```ribalta
@page 16:9

# Abertura
```

O mesmo código aparece para `width` e `height` inválidos no separador, com as mensagens "largura de página inválida" e "altura de página inválida", como em `--- {width=10}` ou `--- {height=alta}`.

**Como corrigir:** informe largura x altura, como `@page 1920x1080` (16:9), `@page 1440x1080` (4:3), `@page 800xauto` ou `@page width=800 height=auto`. Veja [Tamanhos e documentos](/docs/guides/pages/).

### R516 · Nome de cor inválido

**Erro.** Mensagem: nome de cor inválido: `Marca`.

Tokens de cor começam com letra minúscula e usam apenas letras minúsculas, números e hífens.

```ribalta
@color Marca #ff6b4a

# Abertura
```

**Como corrigir:** use um nome como `marca` ou `marca-escura`.

### R517 · Nome de token de fonte inválido

**Erro.** Mensagem: nome de fonte inválido: `Titulo`.

O nome do token (a primeira palavra depois de `@font`) segue a mesma regra dos tokens de cor.

```ribalta
@font Titulo "Bebas Neue" 400

# Abertura
```

**Como corrigir:** use um nome como `titulo`, e aplique com `{font=titulo}`.

### R518 · Nome de estilo inválido

**Erro.** Mensagem: nome de estilo inválido: `Destaque`.

```ribalta
@style Destaque {color=accent weight=bold}

# Abertura
```

**Como corrigir:** use letras minúsculas, números e hífens, como `destaque`.

### R519 · Páginas no tema do projeto

**Erro.** Mensagem: o tema do projeto só pode ter definições, não páginas.

Aparece apenas ao compilar o [tema do projeto](/docs/guides/themes/), nunca nos documentos. O exemplo abaixo é o texto do tema do projeto.

```ribalta
@theme ink
@color marca #12a37f

# Abertura
```

**Como corrigir:** deixe no tema apenas linhas que começam com `@` (como `@theme`, `@page`, `@color`, `@font`, `@style` e `@component`), além de linhas vazias e comentários. O conteúdo das páginas vai em cada documento.

### R520 · `@title` ou `@author` no tema do projeto

**Aviso.** Mensagem: `@title` no tema do projeto é ignorado; defina-o em cada documento.

Aparece apenas ao compilar o tema do projeto. Título e autor são de cada documento, então são ignorados no tema.

```ribalta
@title Relatório anual
@theme ink
```

**Como corrigir:** remova a linha do tema e escreva `@title` ou `@author` no início de cada documento.

### R521 · `@header`, `@footer` ou `@overlay` repetido

**Erro.** Mensagem: `@footer` foi definido mais de uma vez.

Cada documento tem um cabeçalho, um rodapé e uma camada livre. O segundo bloco é ignorado.

```ribalta
@footer
Confidencial
@end

@footer
$page / $pages
@end

# Abertura
```

**Como corrigir:** junte o conteúdo em um único bloco, por exemplo com `::: columns`. Veja [Cabeçalho e rodapé](/docs/guides/header-footer/).

### R522 · Campo desconhecido no cabeçalho, rodapé ou camada livre

**Erro.** Mensagem: `@footer` usa `$pagina`, que não é um campo disponível.

Dentro de `@header`, `@footer` e `@overlay`, `$` seguido de um nome é um campo automático. A dica sugere o campo mais parecido.

```ribalta
@footer
Página $pagina
@end

# Abertura

---
# Agenda
```

**Como corrigir:** use um dos campos: `$page`, `$pages`, `$title`, `$author` ou `$date`.

### R523 · Revelação passo a passo no cabeçalho, rodapé ou camada livre

**Aviso.** Mensagem: `@footer` aparece sempre inteiro; `step` e `::: steps` são ignorados nele.

```ribalta
@footer
::: steps
- Primeiro
- Segundo
:::
@end

# Abertura

---
# Agenda
```

**Como corrigir:** tire `step` e `::: steps` do bloco. Para revelar algo aos poucos, faça isso dentro da página.

### R524 · Notas no cabeçalho, rodapé ou camada livre

**Aviso.** Mensagem: notas (`::: notes`) dentro de `@footer` são ignoradas.

```ribalta
@footer
Confidencial
::: notes
Lembrar de citar o contrato.
:::
@end

# Abertura

---
# Agenda
```

**Como corrigir:** escreva `::: notes` dentro da página a que a nota se refere.

### R525 · Texto na linha do `@header`, `@footer` ou `@overlay`

**Aviso.** Mensagem: o texto `Confidencial` depois de `@footer` será ignorado.

A linha do `@header`, `@footer` ou `@overlay` só aceita atributos entre chaves. O conteúdo vai nas linhas seguintes, até `@end`.

```ribalta
@footer Confidencial
@end

# Abertura
```

**Como corrigir:** passe o texto para a linha de baixo. Para desligar o rodapé que vem do tema do projeto, use `@footer none`.

### R530 · Definição que não cabe no tipo de item de biblioteca

**Erro.** Mensagem: `@theme` não pode ser usado em uma paleta.

Programas que guardam temas, componentes, estilos e paletas reutilizáveis pedem ao compilador para validar cada item de biblioteca. Cada tipo de item aceita só algumas definições: paletas levam `@color` e `@font`; estilos levam também `@style`; componentes levam também `@component`; temas aceitam qualquer definição de tema.

```ribalta
@theme paper
@color marca #349ef4
```

**Como corrigir:** apague a linha apontada ou guarde o item com outro tipo, por exemplo tema em vez de paleta.

### R531 · Item de biblioteca sem definição principal

**Erro.** Mensagem: escolha qual componente é o principal.

Componentes e estilos de biblioteca precisam dizer qual definição é a principal, aquela que quem usar o item vai chamar.

**Como corrigir:** informe qual definição do item é a principal.

### R532 · Definição principal não encontrada

**Erro.** Mensagem: componente não encontrado(a): `cartao`.

A definição principal escolhida não existe no texto do item, ou foi renomeada.

**Como corrigir:** escolha outra definição principal ou corrija o nome no texto.

### R533 · Tipo de item desconhecido

**Erro.** Mensagem: tipo de item desconhecido: `slide`.

O item foi validado com um tipo que o compilador não conhece. Os tipos válidos são tema, componente, estilo e paleta (modelos são validados como tema).

**Como corrigir:** valide o item com um dos tipos da lista.

### R534 · Modo desconhecido

**Erro.** Mensagem: modo desconhecido: `jogo`.

```ribalta
@mode jogo

# Abertura
```

**Como corrigir:** use `@mode default` (o padrão) ou `@mode game`.

## F6xx: variáveis, expressões e estado

### R601 · Variável desconhecida

**Erro.** Mensagem: variável desconhecida: `$totl`.

Uma expressão, `if`, `for` ou `click` usa um nome que não foi declarado com `@let`, `@data` ou `@state`, ou um campo que o registro não tem. Fora de expressões, um `$nome` não declarado continua como texto.

```ribalta
@let total = 10

::: if $totl > 5
Muitos itens
:::
```

**Como corrigir:** corrija o nome (a dica sugere o mais próximo) ou declare a variável.

### R602 · Tipos incompatíveis

**Erro.** Mensagem: esperava número, mas o valor é texto.

O valor não tem o tipo que a operação espera: uma conta com texto, uma condição que não é verdadeiro/falso, um campo em algo que não é registro ou um `@state` recebendo outro tipo.

```ribalta
@let nome = Ana

# $($nome * 2)
```

**Como corrigir:** use valores do tipo certo. Para condições, compare: `$total > 0`.

### R603 · Expressão inválida

**Erro.** Mensagem: a expressão terminou antes do esperado.

A expressão está mal escrita: falta um valor, um parêntese ou aspas, ou há um caractere que não faz parte da linguagem.

```ribalta
# Total: $(10 +)
```

**Como corrigir:** complete a expressão. Para comparar, use `==` (não `=`).

### R604 · Valor que depende do estado

**Erro.** Mensagem: este valor depende do estado do documento e não pode ser usado aqui.

Valores de `@state` (e `$step`) só podem ser usados em condições (`if`), variantes (`.estilo=condição`), texto e `click`. Atributos visuais, `@let` e listas de `for` precisam de valores fixos.

```ribalta
@state modo = claro

# Olá {color=$modo}
```

**Como corrigir:** use uma variante: `@style escuro {color=ink}` e `{.escuro="$modo == escuro"}`.

### R605 · Variável que não pode mudar

**Erro.** Mensagem: `$total` não pode ser alterado.

Um `click` tenta mudar um `@let`, um `@data`, um parâmetro de componente ou uma variável do `for`. Só `@state` muda enquanto o documento é mostrado.

```ribalta
@let total = 0

[Somar]{click="$total = $total + 1"}
```

**Como corrigir:** declare a variável com `@state total = 0`.

### R606 · Repetição inválida

**Erro.** Mensagem: `for` precisa de uma lista, mas o valor é número.

O `for` não recebeu uma lista, está mal escrito, depende do estado ou passaria do limite de 500 repetições no documento.

```ribalta
@let total = 3

::: for item in $total
$item
:::
```

**Como corrigir:** use uma lista (`@data`, `[a, b]`) ou um intervalo (`1..3`).

### R607 · `else` sem `if`

**Erro.** Mensagem: `else` precisa vir logo depois de um bloco `if`.

`::: else` e `::: else if` precisam vir imediatamente depois de um `::: if` (ou de outro `else if`) no mesmo bloco. Outro bloco no meio, ou um segundo `else`, quebra a sequência.

```ribalta
# Título
::: else
Alternativa
:::
```

**Como corrigir:** coloque o `::: else` logo depois do `::: if … :::` correspondente.

### R608 · Valor que o estado nunca recebe

**Aviso.** Mensagem: `$plano` nunca recebe o valor `prro`.

Uma condição compara um `@state` de texto com um valor que não é o inicial nem é atribuído por nenhum `click`. Geralmente é um erro de digitação.

```ribalta
@state plano = pro

::: if $plano == prro
Plano Pro
:::
```

**Como corrigir:** corrija o valor (a dica mostra os valores usados).

### R609 · Definição inválida

**Erro.** Mensagem: `@let` espera um nome, `=` e um valor.

Um `@let` ou `@state` sem `=` ou com nome inválido, um `@data` sem nome ou sem tabela, ou um campo de `@data` com nome inválido.

```ribalta
@let destaque coral
```

**Como corrigir:** escreva `@let destaque = coral`. Nomes usam letras minúsculas, números, hífens e sublinhados.

### R610 · Nome repetido

**Erro.** Mensagem: `$total` já foi definido.

`@let`, `@data` e `@state` compartilham os mesmos nomes, que precisam ser únicos. `step` é reservado para a etapa atual.

```ribalta
@let total = 1
@state total = 2
```

**Como corrigir:** dê nomes diferentes às variáveis.

### R611 · Ação inválida

**Erro.** Mensagem: ação inválida: `pular`.

O valor de `click` não é uma ação conhecida.

```ribalta
[Próximo]{click="pular"}
```

**Como corrigir:** use `$x = valor`, `toggle $x`, `go #id`, `go back`, `go next` ou `go prev`, separadas por `;`.

### R612 · Navegação em `on-enter`

**Erro.** Mensagem: `on-enter` só pode mudar o estado; `go`, `back`, `next` e `prev` não são permitidos.

`on-enter` roda ao entrar na página. Mudar de página ali levaria a outra página sem ninguém pedir, e poderia criar um ciclo.

```ribalta
--- {on-enter="go #fim"}
# Introdução
```

**Como corrigir:** deixe em `on-enter` só `$x = valor` e `toggle $x`. Para escolher a página seguinte, use `next=#id` no separador.

### R613 · Sorteio fora de uma ação

**Erro.** Mensagem: `random` só pode ser usado em ações (`click` e `on-enter`).

`random` e `pick` sorteiam um valor novo a cada chamada. No texto, numa condição ou num `@let`, o resultado mudaria a cada vez que a tela é desenhada.

```ribalta
@state dado = 0

# Sua vez
Você tirou $(random(1, 6))
```

**Como corrigir:** sorteie numa ação e guarde o resultado num `@state`: `[Rolar]{click="$dado = random(1, 6)"}` e `Você tirou $dado`.

### R614 · `on-timeout` sem `timeout`

**Erro.** Mensagem: `on-timeout` precisa de `timeout` na mesma página.

```ribalta
# Abertura

--- {on-timeout="go next"}
# Introdução
```

**Como corrigir:** diga quantos segundos esperar no mesmo separador: `--- {timeout=30 on-timeout="go next"}`.

### R615 · Som desconhecido

**Erro.** Mensagem: esta página não tem o som `prta`.

`play` e `stop` só alcançam os sons declarados com `:: sound` na mesma página.

```ribalta
# A porta
:: sound https://exemplo.com/porta.mp3 {name=porta}
[Abrir]{click="play prta"}
```

**Como corrigir:** corrija o nome ou declare o som na página com `{name=…}`.

### R616 · Som repetido

**Erro.** Mensagem: já existe outro som `porta` nesta página.

**Como corrigir:** dê um nome diferente a cada `:: sound` da página.

### R617 · `@persist` com algo que não é estado

**Erro.** Mensagem: `@persist` só guarda variáveis de @state, e `$fixo` não é uma.

```ribalta
@let fixo = 1
@persist fixo

# Abertura
```

**Como corrigir:** liste só variáveis declaradas com `@state` no início do documento.

### R618 · `@scorm` inválido

**Erro.** Mensagem: `score` em @scorm precisa ser uma variável numérica de @state, e `$nome` não é.

```ribalta
@state nome = "Ana"
@scorm score=$nome

# Abertura
```

Também aparece com parâmetro desconhecido (só valem `score`, `complete`, `max` e `pass`), com `max` ou `pass` que não são números positivos, com `pass` maior que `max` e quando faltam `score` e `complete`.

**Como corrigir:** use um `@state` numérico como nota, como `@state pontos = 0` e `@scorm score=$pontos complete=#fim`.

### R619 · Mudança num valor de fora

**Erro.** Mensagem: `$rodada` vem de fora do documento (@extern) e não muda por ações.

```ribalta
@extern rodada = 1

# Placar
[Próxima]{click="$rodada = $rodada + 1"}
```

Também aparece em `:: input {bind=$rodada}`, em `@persist rodada` e em `@scorm score=$rodada`.

**Como corrigir:** deixe quem mostra o documento definir o valor, ou declare com `@state` se o próprio documento deve mudá-lo.

### R620 · Item de repetição numa ação

**Erro.** Mensagem: ações não podem usar `$coisa`, que só existe enquanto a repetição é desenhada.

```ribalta
@state mochila = [mapa, chave]
@state escolhida = ""

# Mochila
::: for coisa in $mochila
[$coisa]{click="$escolhida = $coisa"}
:::
```

Aparece quando um `::: for` percorre uma lista que muda enquanto o documento é mostrado (`@state` ou `@extern`) e uma ação usa o item ou a posição.

**Como corrigir:** use o item no texto, em `if` ou em variantes; para ações, use valores do `@state`.

## F7xx: navegação

### R701 · Página inexistente

**Erro** em `click="go #id"` e `next=#id`; **aviso** em links `[texto](#id)`. Mensagem: não existe página com o id `#fim`.

```ribalta
[Ir para o fim]{click="go #fim"}
```

**Como corrigir:** dê o id à página de destino (`--- {#fim}`) ou corrija o nome.

### R702 · Página oculta inalcançável

**Aviso.** Mensagem: a página oculta `#extra` não é alcançada por nenhum link ou `go`.

Uma página com `hidden` não aparece no fluxo normal; sem um link, `go` ou `next` apontando para ela, ninguém consegue vê-la.

```ribalta
# Início
--- {#extra hidden}
Detalhes
```

**Como corrigir:** crie um link para a página (`[Detalhes](#extra)`) ou remova o `hidden`.

## F8xx: arquivos externos

Só aparecem na [linha de comando](/docs/reference/cli/), com `ribalta check --check-assets`, que acessa a internet para conferir as imagens e os sons.

### R801 · Imagem ou som inacessível

**Aviso.** Mensagem: a imagem `https://exemplo.com/foto.jpg` respondeu 404.

```ribalta
# Equipe

![Foto da equipe](https://exemplo.com/foto.jpg)
```

Também aparece quando o endereço não responde (domínio errado, sem conexão) e quando ele é uma página, não o arquivo: por exemplo, o link de uma foto no Unsplash em vez do endereço da imagem.

**Como corrigir:** abra o endereço no navegador. Se for uma página, clique com o botão direito na imagem e copie o endereço dela. Se o arquivo não existir mais ou exigir login, envie a imagem para o projeto.
