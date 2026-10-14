---
title: "Interatividade e navegação"
description: "Use @state, cliques, variantes e go para criar documentos que reagem a quem os usa e seguem caminhos diferentes."
---

Alguns documentos não são lineares: você quer abrir um detalhe só se alguém perguntar, comparar dois cenários com um clique ou pular para uma seção e voltar. A Ribalta faz isso sem JavaScript, com uma camada pequena de **estado**:

| Recurso | Para quê |
| --- | --- |
| `@state` | Uma variável que muda enquanto o documento é mostrado. |
| `click="…"` | Ações ao clicar: mudar o estado, ir para outra página, voltar. |
| `if` | Mostrar ou esconder conteúdo conforme o estado. |
| `.estilo=condição` | Aplicar um estilo só quando a condição é verdadeira. |
| `$nome` no texto | Mostrar o valor atual de uma variável. |
| `@state x = []` | Uma lista de textos que muda enquanto o documento é mostrado, como um inventário. |
| `random`, `pick` | Sortear um número ou um item dentro de uma ação. |
| `$elapsed`, `timeout` | Contar o tempo na página e agir quando ele acaba. |
| `:: input` | Um campo de texto para quem assiste digitar. |
| `key=…` | Uma tecla que aciona o clique de um elemento. |
| `:: sound`, `play` | Sons e efeitos sonoros. |
| `@persist` | Guardar o progresso no navegador. |
| `@extern` | Um valor que vem de quem mostra o documento, como o placar de um jogo. |
| `--- {hidden}` | Páginas fora do fluxo normal, alcançadas só por link ou `go`. |
| `--- {on-enter="…"}` | Mudar o estado sempre que uma página é aberta. |
| `--- {nav=off}`, `@mode game` | Páginas que só se deixam pelas opções, para jogos e escolhas. |

## Estado com `@state`

Declare a variável e o valor inicial no início do documento:

```ribalta
@state detalhes = false
@state plano = pro
@state cliques = 0
```

O valor inicial pode ser verdadeiro/falso, número ou texto. Diferente de `@let`, o valor de um `@state` muda enquanto o documento é mostrado: ao clicar em um elemento, ao navegar ou ao entrar em uma página.

## Cliques

O atributo `click` recebe uma ou mais ações, separadas por `;`:

```ribalta preview
@state detalhes = false

# Por que o custo cai?
[Mostrar detalhes]{click="toggle $detalhes" .accent}
::: if $detalhes
::: box {.card enter=up}
A renderização acontece no navegador, então não há servidores de exportação.
:::
:::
```

| Ação | Efeito |
| --- | --- |
| `$x = valor` | Muda o valor. O tipo precisa ser o mesmo do `@state` ([R602](/docs/reference/errors/)). |
| `toggle $x` | Inverte verdadeiro/falso. |
| `add $lista valor` / `remove $lista valor` | Põe ou tira um texto de uma lista. Veja [Listas no estado](#listas-no-estado). |
| `clear $lista` | Esvazia a lista. |
| `go #id` | Vai para a página com esse id. |
| `go back` | Volta para onde você estava antes do último `go` ou link. |
| `go next` / `go prev` | Avança ou volta uma etapa, como as setas. |

`click` funciona em qualquer bloco (`::: box {click="…"}`) e em trechos de texto (`[texto]{click="…"}`). Clicar num elemento interativo não avança a página. Só variáveis de `@state` podem mudar ([R605](/docs/reference/errors/)).

## Conteúdo que depende do estado

Use `::: if`, `::: else` e o atributo `if` exatamente como em [Variáveis, dados e repetição](/docs/guides/data/#condições-com--if). A diferença é que, com `@state`, todas as alternativas vão para o documento e a escolha acontece na hora:

```ribalta preview
@state cenario = otimista

## Receita em 2027
[Otimista]{click="$cenario = otimista"} · [Conservador]{click="$cenario = conservador"}
::: if $cenario == otimista
# R$ 20 milhões {color=green}
:::
::: else
# R$ 14 milhões {color=coral}
:::
```

Um elemento escondido por `if` sai do layout e o resto da página se reorganiza. Já um elemento com `step` reserva o espaço até aparecer.

## Variantes: estilo conforme o estado

`.estilo=condição` aplica o `@style` só quando a condição é verdadeira. É o jeito de destacar o item escolhido:

```ribalta preview
@state plano = pro
@style escolhido {border="4 accent" shadow=lg}
@style apagado {opacity=0.45}

::: grid {cols=3 gap=lg}
::: box {.card .escolhido="$plano == basic" .apagado="$plano != basic" click="$plano = basic"}
### Básico
:::
::: box {.card .escolhido="$plano == pro" .apagado="$plano != pro" click="$plano = pro"}
### Pro
:::
::: box {.card .escolhido="$plano == team" .apagado="$plano != team" click="$plano = team"}
### Equipes
:::
:::
```

Atributos visuais como `color`, `size` e `w` só aceitam valores fixos. Usar `@state` neles é erro ([R604](/docs/reference/errors/)): use uma variante. Assim o compilador valida todas as cores e medidas antes de mostrar o documento.

### Ao passar o mouse

`.estilo=hover` aplica o estilo enquanto o ponteiro do mouse está sobre o elemento. É um bom sinal de que algo é clicável:

```ribalta preview
@style realce {border="3 accent" shadow=lg}

::: grid {cols=2 gap=lg}
::: box {.card .realce=hover click="go next"}
### Seguir pela floresta
:::
::: box {.card .realce=hover click="go next"}
### Voltar para a vila
:::
:::
```

- Funciona em blocos, itens de lista e trechos de texto, e pode ser combinado com outras variantes. As mudanças de cor, fundo, borda, sombra e opacidade são animadas.
- Em telas de toque não existe "passar o mouse", então o estilo não aparece por lá: não use `hover` para mostrar informação importante.
- A condição é exatamente `hover`; para combinar com o estado, use duas variantes, como `{.realce=hover .escolhido="$plano == pro"}`.

## Valores no texto

`$nome` e `$( … )` mostram o valor atual:

```ribalta preview
@state anual = false

## Plano Pro
R$ $($anual ? 490 : 49) {size=4xl weight=black}
$($anual ? "por ano" : "por mês") {.muted}
[Cobrança anual: **$($anual ? "ligada" : "desligada")**]{click="toggle $anual"}
```

## Listas no estado

Um `@state` também pode guardar uma lista de textos: os finais que alguém já descobriu, os itens de uma mochila, as perguntas respondidas. A lista não repete itens e mantém a ordem em que eles entraram.

```ribalta preview
@state mochila = [mapa]

# A porta trancada
Você carrega: $mochila ($(len($mochila)) itens)

A porta abre. {if="chave in $mochila"}

[Pegar a chave]{click="add $mochila chave"} · [Largar o mapa]{click="remove $mochila mapa"} · [Esvaziar]{click="clear $mochila"}
```

| Recurso | Efeito |
| --- | --- |
| `@state x = []`, `@state x = [a, b]` | Declara a lista, vazia ou com itens iniciais. Os itens são textos. |
| `add $x valor` | Põe o valor no fim da lista, se ele ainda não estiver nela. |
| `remove $x valor` | Tira o valor da lista. |
| `clear $x` | Esvazia a lista. |
| `valor in $x` | Verdadeiro se o valor está na lista. Use `not (valor in $x)` para o contrário. |
| `len($x)` | Quantos itens a lista tem. |
| `$x` no texto | Os itens separados por vírgula. |

As ações de lista funcionam em `click` e em `on-enter`. Com elas, um jogo pode contar os finais descobertos sem uma variável para cada final:

```ribalta
@state finais = []

# Início

--- {#vitoria on-enter="add $finais vitoria"}
# Você venceu
Finais descobertos: $(len($finais)) de 3
```

- `in` também funciona com uma lista fixa: `$rota in [mar, vila]` é o mesmo que `$rota == mar or $rota == vila`.
- Listas do estado não se comparam com `==`; use `in` e `len` ([R602](/docs/reference/errors/)). Elas também não mudam com `=`: use `add`, `remove` e `clear`.
- A Ribalta avisa quando `valor in $x` procura um texto que nenhuma ação adiciona, como um erro de digitação ([R608](/docs/reference/errors/)).
- `::: for item in $x` também percorre listas do estado: o conteúdo é repetido enquanto o documento é mostrado e acompanha cada `add` e `remove`. Dentro dele, ações não usam `$item` ([R620](/docs/reference/errors/)).

## Sorteios

`random(menor, maior)` sorteia um número inteiro entre os dois valores, incluindo os dois. `pick(lista)` sorteia um item de uma lista, fixa ou do estado. Os dois só funcionam dentro de ações (`click` e `on-enter`), e o resultado vai para um `@state`:

```ribalta preview
@state dado = 0
@state jogada = nenhuma

# Pedra, papel e tesoura
Dado: $dado · Jogada: $jogada

[Rolar o dado]{click="$dado = random(1, 6)"} · [Jogar]{click="$jogada = pick([pedra, papel, tesoura])"}
```

- Cada chamada faz um sorteio novo: `$dado = random(1, 6) + random(1, 6)` soma dois dados.
- Com uma lista do estado, dá para tirar cartas sem repetir: `click="$carta = pick($baralho); remove $baralho $carta"`.
- Fora de uma ação, como em `$(random(1, 6))` no texto ou num `if`, o número mudaria a cada vez que a tela é desenhada, então a Ribalta recusa ([R613](/docs/reference/errors/)). Guarde o sorteio num `@state` e mostre a variável.

## Tempo

`$elapsed` são os segundos desde que a página abriu: 0, 1, 2… Ele volta a zero sempre que você entra na página. Com ele, dá para mostrar uma contagem regressiva e revelar algo depois de um tempo. `timeout` e `on-timeout` no separador rodam ações quando o tempo acaba, como um clique automático:

```ribalta preview
@state pontos = 0

--- {timeout=30 on-timeout="go #esgotado"}
# Qual é a capital do Brasil?
Restam $(30 - $elapsed) segundos

Dica: começa com B {if="$elapsed >= 15"}

[Brasília]{click="$pontos = $pontos + 1; go #acertou"} · [Rio de Janeiro]{click="go #esgotado"}

--- {#acertou hidden}
# Acertou!

--- {#esgotado hidden}
# Tempo esgotado
```

- `on-timeout` aceita as mesmas ações de `click`, inclusive `go`. `--- {timeout=10 on-timeout="go next"}` avança sozinho, bom para um quiosque.
- O relógio só corre enquanto o documento é mostrado de forma interativa. Numa visualização estática, como uma imagem ou um PDF, `$elapsed` fica em 0.
- Voltar a uma página com `go back` recomeça a contagem.

## Entrada de texto

`:: input {bind=$nome}` mostra um campo de texto ligado a um `@state`. Com ele, a história chama a pessoa pelo nome, um quiz recebe respostas escritas e uma porta pede senha:

```ribalta preview
@state senha = ""

# A porta pede uma senha
:: input {bind=$senha placeholder="Senha" on-submit="go #dentro"}

A porta range… {if="$senha == abracadabra"}

--- {#dentro hidden}
# Você entrou
```

- `on-submit` roda ações quando a pessoa aperta Enter, inclusive `go`. Sem `on-submit`, o Enter no campo aciona o elemento da página com `key=enter`, como o botão de enviar de um formulário.
- O valor muda a cada tecla. Para aceitar variações de uma resposta, use uma lista: `{if="$resposta in [Brasília, brasília, brasilia]"}`.
- Digitar no campo não avança a página, e as setas movem o cursor dentro dele.
- Detalhes em [Blocos › input](/docs/reference/blocks/#input).

## Teclado

`key` dá a um elemento clicável uma tecla de atalho. Apertar a tecla roda as mesmas ações do `click`, como num jogo de mover pelo mapa:

```ribalta preview
@state x = 0
@state y = 0

--- {nav=off}
# Posição: $x, $y
[↑]{click="$y = $y - 1" key=up} [↓]{click="$y = $y + 1" key=down} [←]{click="$x = $x - 1" key=left} [→]{click="$x = $x + 1" key=right}

[Recomeçar]{click="$x = 0; $y = 0" key=r}
```

- Teclas aceitas: uma letra, um número, `up`, `down`, `left`, `right` (as setas), `Enter` e `Space`.
- Uma tecla usada num elemento deixa de navegar naquela página: com `key=right`, a seta direita move o personagem em vez de avançar. Combine com `nav=off` quando a página inteira for um jogo.
- Só valem os elementos que estão aparecendo: um `if` falso ou uma etapa ainda não revelada desligam a tecla.
- Todo elemento clicável também pode ser alcançado com **Tab** e acionado com **Enter** ou **espaço**, para quem navega sem mouse. Uma variante `.estilo=hover` também aparece quando o elemento recebe o foco pelo teclado.

## Som

`:: sound` declara um som na página, e as ações `play` e `stop` o controlam. Os sons param quando você sai da página:

```ribalta
--- {on-enter="play chuva"}
:: sound https://exemplo.com/chuva.mp3 {name=chuva loop volume=40%}
:: sound https://exemplo.com/porta.mp3 {name=porta}
# A porta
[Abrir a porta]{click="play porta; go next"}
```

- `play` e `stop` funcionam em `click`, `on-enter`, `on-timeout` e `on-submit`. `play` recomeça o som do início.
- `autoplay` toca o som ao entrar na página, sem precisar de ação.
- O som só toca quando o documento é mostrado de forma interativa. Visualizações estáticas, como imagens e PDF, ficam mudas, e o programa que mostra o documento pode silenciar telas extras para o som não tocar duas vezes.
- Os navegadores só tocam som depois de algum clique ou tecla; na primeira página mostrada, o `autoplay` pode esperar a primeira interação.
- Use endereços que comecem com `https://`.

## Progresso salvo

`@persist` lista os `@state` que ficam guardados no navegador de quem assiste. Ao fechar a página e voltar, o progresso continua de onde parou:

```ribalta
@state finais = []
@state moedas = 0
@persist finais moedas
```

- Só valem `@state` do início do documento ([R617](/docs/reference/errors/)). As outras variáveis começam do valor inicial, como sempre.
- O progresso fica no navegador de cada pessoa, separado para cada endereço.
- Para recomeçar do zero, use as próprias ações: `click="clear $finais; $moedas = 0"`.

## Nota e conclusão em LMS (SCORM)

Treinamentos e quizzes podem rodar dentro de um LMS (Moodle, Canvas, TalentLMS…) e mandar a nota e a conclusão para ele. Diga qual `@state` é a nota e em quais páginas o aluno conclui:

```ribalta
@state pontos = 0
@scorm score=$pontos complete=#fim pass=70

--- {#inicio}
# Qual senha é mais forte?
[Abacate-Lua-Trem-42]{click="$pontos = $pontos + 100; go #fim"}
[123456]{click="go #fim"}

--- {#fim hidden}
# Você fez $pontos pontos
```

Depois gere o pacote com a [linha de comando](/docs/reference/cli/) e envie o `.zip` ao LMS como pacote SCORM:

```sh
ribalta build quiz.ribalta --target scorm
```

| Parâmetro | O que faz |
| --- | --- |
| `score=$variavel` | `@state` numérico enviado como nota, sempre que muda. |
| `complete=#pagina` | Página que conclui o curso ao ser alcançada. Vários: `complete=#venceu,#perdeu`. |
| `max=100` | Nota máxima (padrão `100`). A nota enviada fica entre `0` e `max`. |
| `pass=70` | Nota mínima para aprovar. Com ela, a conclusão vira "aprovado" ou "reprovado"; sem ela, "concluído". |

- O pacote usa SCORM 1.2, aceito por praticamente todos os LMS.
- O LMS guarda a sessão: se o aluno sair no meio, volta para a mesma página com o mesmo estado.
- Sem `@scorm`, o pacote só avisa a conclusão quando o aluno chega à última página.
- Fora de um LMS, `@scorm` não faz nada.

## Valores de fora com `@extern`

Às vezes o documento mostra algo que ele mesmo não decide: o placar de um jogo que roda num servidor, a pergunta da rodada, a cotação do dia. `@extern` declara um valor assim. Quem mostra o documento, como um app que embute a Ribalta, define o valor e pode trocá-lo a qualquer momento, e as páginas se redesenham na hora.

```ribalta preview
@extern pergunta = "Qual é a capital do Brasil?"
@extern rodada = 3
@extern revelada = false

Pergunta $rodada {.kicker}
# $pergunta
Brasília {if="$revelada" color=green}
```

- O valor depois do `=` é o exemplo: define o tipo (verdadeiro/falso, número, texto ou lista de textos) e é o que aparece quando ninguém fornece o valor, como numa prévia ou num PDF.
- Use `$nome`, `if`, variantes e expressões como com `@state`.
- Só quem mostra o documento muda um `@extern`: ações, `:: input`, `@persist` e o `score` de `@scorm` não aceitam ([R619](/docs/reference/errors/)). Para um valor que o próprio documento muda, use `@state`.

### Listas de registros

Um `@extern` também pode ser uma lista de registros, como o placar de um jogo. O exemplo é uma tabela `@data`, que define os campos e o que aparece sem host; `::: for` repete o conteúdo para cada linha que chegar:

```ribalta preview
@data exemplo
| nome  | pontos |
| ---   | ---    |
| Ana   | 300    |
| Bruno | 200    |
@end
@extern placar = $exemplo

## Placar
::: for jogador, i in $placar
$i. **$jogador.nome**: $jogador.pontos pontos
:::
```

- A tabela de exemplo precisa de pelo menos uma linha: ela dá o tipo de cada campo.
- Quem mostra o documento pode enviar mais ou menos linhas; o `for` acompanha.
- Dentro do `for`, use `$jogador.campo` no texto, em `if` e em variantes. Ações não usam o item ([R620](/docs/reference/errors/)), e atributos visuais como `size` continuam só com valores fixos.
- `len($placar)` dá o número de linhas.

## Navegação

### Links e `go`

Um link com `#id` vai para a página com esse id. `click="go #id"` faz o mesmo e pode ser combinado com outras ações; elas são executadas em ordem, então a página de destino já aparece com o novo estado:

```ribalta
--- {#precos}
## Escolha um plano
[Plano Pro →]{click="$plano = pro; go #detalhe"}
[Plano Equipes →]{click="$plano = team; go #detalhe"}
```

### Desvios com `hidden`

Uma página com `hidden` fica fora do fluxo normal: as setas pulam essa página e ela só é alcançada por link ou `go`. Quando você avança depois da última etapa de uma página oculta, volta para onde estava.

```ribalta preview
@state plano = pro

# Escolha um plano
[Ver o Pro](#detalhe)
--- {#detalhe hidden}
::: if $plano == pro
## Pro: até 25 usuários
:::
[← Voltar]{click="go back"}
```

Uma página oculta que nenhum link ou `go` alcança gera um aviso ([R702](/docs/reference/errors/)).

### Mudando a próxima página

`--- {next=#id}` troca a página seguinte quando você avança, para criar caminhos alternativos sem duplicar o documento.

| Tipo | Como | O que acontece |
| --- | --- | --- |
| Normal | setas, espaço, clique na página | Próxima etapa ou próxima página visível. |
| Salto | `[x](#id)`, `click="go #id"` | Vai à página e guarda a posição de origem. |
| Com estado | `click="$x = v; go #id"` | Muda o estado e depois vai à página. |
| Volta | `click="go back"` | Retorna à posição guardada pelo último salto. |
| Condicional | dois botões com `if`, ou `next=#id` | A condição fica no elemento, não na ação. |

## Estado dentro de componentes

Um componente lê e altera direto os `@state` declarados no início do documento: `$pontos` no texto, `if=$aberto` e `click="$pontos = $pontos + 1"` funcionam dentro do `@component` como em qualquer página.

Para o mesmo componente servir a variáveis diferentes, passe o `@state` como parâmetro (`estado=$q1`): dentro dele, `$estado = feito` muda o `$q1` original. Destinos de `go` também aceitam parâmetros, como `go #pergunta-$n`. Assim um único componente atende vários itens:

```ribalta preview
@state q1 = pendente
@state q2 = pendente
@component item {n estado}
[Item $n: $estado]{click="$estado = feito"}
@end

:: item {n=1 estado=$q1}
:: item {n=2 estado=$q2}
```

O `@state` de uma página (veja abaixo), o `@let` de uma página e as variáveis de `::: for` só existem onde o componente é usado, então o componente não os enxerga: passe-os como parâmetro ([R410](/docs/reference/errors/)).

## Estado de uma página

Um `@state` no topo de uma página, logo depois do `---`, só existe nessa página. Ele volta ao valor inicial sempre que você entra na página, e `go back` restaura o valor que estava:

```ribalta preview
# Início
--- {#demo}
@state aberto = false
## Demonstração
[Abrir]{click="toggle $aberto"}
Conteúdo da demonstração {if=$aberto}
```

## Ação ao entrar na página

`on-enter` no separador roda ações sempre que a página passa a ser a atual: ao começar a mostrar o documento, ao avançar ou voltar com as setas, por `go`, link ou `next`. Ele não roda de novo quando você volta com `go back`, porque isso é retornar a um ponto em que a página já foi aberta.

```ribalta preview
@state visitas = 0
@state viu_o_final = false

# Início
Esta é a página de abertura.

--- {on-enter="$visitas = $visitas + 1"}
# Meio
Você já passou por aqui $visitas vez(es).

--- {on-enter="$viu_o_final = true"}
# Fim
```

- As ações são as mesmas de `click` que mudam o estado: `$x = valor` e `toggle $x`, separadas por `;`. Ir para outra página (`go`, `go back`, `go next`, `go prev`) não é permitido ([R612](/docs/reference/errors/)); para escolher a página seguinte, use `next=#id`.
- O estado da própria página volta ao valor inicial antes das ações, então `on-enter` também pode ajustá-lo.
- `on-enter` funciona dentro de um `@style`, o que permite reaproveitar a mesma ação em várias páginas. As ações do estilo rodam antes das escritas no separador:

```ribalta
@state finais = 0
@style final {on-enter="$finais = $finais + 1"}

--- {#vitoria .final}
# Você venceu
Finais descobertos: $finais
```

## Páginas sem avanço e modo jogo

Num jogo ou numa história de escolhas, quem decide a próxima página é a opção clicada, não a seta. Com `nav=off` no separador, as setas, o espaço, o clique na página e o deslizar não saem dela:

```ribalta
@state rota = nenhuma

--- {nav=off}
# A encruzilhada
- Uma pista aparece {step=1}
[Ir para o mar]{click="$rota = mar; go #mar"}
[Ir para a vila]{click="$rota = vila; go #vila"}

--- {#mar hidden}
# O mar
[Voltar]{click="go back"}

--- {#vila hidden}
# A vila
[Voltar]{click="go back"}
```

- As etapas da página continuam sendo reveladas e escondidas pelas setas.
- As ações escritas na página continuam funcionando, inclusive `go next` e `go prev`.
- Numa página `hidden` aberta por `go`, a seta também não volta sozinha para onde você estava. Para isso, ofereça um `[Voltar]{click="go back"}`.

`@mode game` no início do documento (ou no tema do projeto) faz isso em todas as páginas. Ele também avisa ao programa que mostra o documento que se trata de um jogo, para ele esconder o que entregaria o final antes da hora, como o contador de páginas ou um atalho para a última página. O modo padrão é `@mode default`. Uma página pode voltar ao normal com `nav=on`:

```ribalta
@mode game

# A Última Luz
[Começar]{click="go #capitulo-1"}

--- {#capitulo-1}
# Capítulo 1

--- {nav=on}
# Créditos
```

## Etapas com `$step`

`$step` é a etapa atual da página. Com ela, uma revelação pode **substituir** em vez de só acumular, como apagar os itens anteriores:

```ribalta preview
@let etapas = [Importar, "Ajustar o tema", Compartilhar]
@style apagado {opacity=0.35}

## Como começar
::: for t, i in $etapas
1. $t {step=$i .apagado="$step > $i"}
:::
```

O número de etapas da página considera `step`, `::: steps` e os números comparados com `$step`. Para fixar outro valor, use `--- {steps=4}`.

## Animações

| Atributo | Onde | Valores | Efeito |
| --- | --- | --- | --- |
| `enter` | Qualquer bloco ou trecho | `fade`, `up`, `down`, `left`, `right`, `zoom`, `none` | Como o elemento aparece numa etapa ou quando um `if` passa a ser verdadeiro. O padrão das etapas é `up`. |
| `transition` | Separador `---` | `fade`, `slide`, `zoom`, `none` | Transição ao entrar na página. |
| `@transition` | Preâmbulo | Os mesmos de `transition` | Transição padrão de todas as páginas. |

Como `enter` é um atributo comum, ele pode morar num `@style` (`@style surge {enter=zoom}`), deixando a animação fora da estrutura da página. Mudanças de variante (cor, fundo, opacidade, sombra) são animadas automaticamente.

## Visualizações estáticas

Uma visualização estática do documento, como uma imagem ou um PDF, usa o estado inicial e mostra todas as etapas.

## Erros comuns

| Código | Quando acontece |
| --- | --- |
| [R604](/docs/reference/errors/) | `@state` usado num atributo visual ou num `@let`. |
| [R605](/docs/reference/errors/) | `click` tentando mudar um `@let`, um dado ou um parâmetro. |
| [R608](/docs/reference/errors/) | Comparação com um valor que o estado nunca recebe, como `$plano == prro`. |
| [R611](/docs/reference/errors/) | Ação de `click` mal escrita. |
| [R612](/docs/reference/errors/) | `on-enter` tentando mudar de página. |
| [R613](/docs/reference/errors/) | `random` ou `pick` fora de uma ação. |
| [R614](/docs/reference/errors/) | `on-timeout` sem `timeout`. |
| [R116](/docs/reference/errors/) | `:: input` sem `bind`. |
| [R210](/docs/reference/errors/) | `key` sem `click`. |
| [R615](/docs/reference/errors/) | `play` ou `stop` com um som que não está na página. |
| [R617](/docs/reference/errors/) | `@persist` com algo que não é um `@state`. |
| [R618](/docs/reference/errors/) | `@scorm` com parâmetro desconhecido, nota que não é um `@state` numérico ou número inválido. |
| [R534](/docs/reference/errors/) | `@mode` com um valor diferente de `default` ou `game`. |
| [R701](/docs/reference/errors/) | `go #id`, link ou `next` para uma página que não existe. |
| [R702](/docs/reference/errors/) | Página `hidden` que ninguém alcança. |
