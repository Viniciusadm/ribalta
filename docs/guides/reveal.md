---
title: Revelação por etapas
description: Mostre o conteúdo de uma página aos poucos, um bloco, item ou trecho por vez.
---

Revelar por etapas ajuda o público a acompanhar o raciocínio: cada ponto aparece quando você chega nele, e não todos de uma vez. Na Ribalta há duas formas de fazer isso: o bloco `::: steps` e o atributo `step`.

## O bloco `steps`

Tudo o que estiver dentro de `::: steps` aparece em ordem, um por vez, a cada avanço:

```ribalta preview
# Por que mudar agora
::: steps {gap=md}
Os custos de operação subiram 18% em um ano.

A concorrência já automatizou o atendimento.

Os clientes pedem respostas no mesmo dia.
:::
```

Cada bloco filho (um parágrafo, um título, uma imagem, outro bloco) é uma etapa.

### Listas

Quando o filho é uma lista, cada **item** vira uma etapa:

```ribalta preview
# Plano para o semestre
::: steps
- Mapear os processos atuais
- Escolher as ferramentas
- Treinar as equipes
- Medir os resultados
:::
```

O bloco `steps` organiza os filhos numa pilha vertical e aceita os mesmos atributos de `::: stack`, como `gap` e `align`.

## Etapas explícitas com `step`

Para controlar a ordem você mesmo, use o atributo `step=N` em qualquer bloco. O bloco aparece na etapa `N`:

```ribalta preview
::: columns {gap=xl}
::: box {.card}
## Hoje
Relatórios montados à mão, uma vez por mês.
:::
::: box {.card step=1}
## Em seis meses
Relatórios automáticos, atualizados todos os dias.
:::
:::
```

O atributo também funciona em:

- **itens de lista**: `- Terceiro ponto {step=3}`;
- **trechos de texto**: `A meta é [dobrar a receita]{step=1 color=accent}.`;
- **componentes**: `:: metrica {valor=38% rotulo="crescimento" step=2}`.

```ribalta preview
# Meta para 2027
A receita foi de R$ 12 milhões. [A meta é chegar a R$ 20 milhões.]{step=1 color=accent weight=bold}
```

O número da etapa vai de 1 a 99. Vários elementos com o mesmo número aparecem juntos.

## Como as etapas se combinam

- Elementos sem etapa ficam visíveis desde o início da página.
- A página tem tantas etapas quanto o maior número usado nela.
- Um `::: steps` continua a contagem a partir da maior etapa que já apareceu antes dele na página. Se um título tem `{step=2}` e logo depois vem um `::: steps` com três itens, os itens ficam nas etapas 3, 4 e 5.
- Dentro de um `::: steps`, a ordem é definida pelo próprio bloco. Não use `step` nos filhos diretos dele.

```ribalta preview
# Resultados {step=1}
::: steps
- Receita: +38%
- Clientes: +94
- Satisfação: 4,8
:::
```

Neste exemplo, o título aparece na etapa 1 e os itens nas etapas 2, 3 e 4.

## Substituindo em vez de acumular

As etapas normais só acrescentam conteúdo. Com `$step`, a etapa atual da página, você pode destacar o item atual, apagar os anteriores ou trocar um conteúdo por outro:

```ribalta preview
@style apagado {opacity=0.35}

## Plano para o semestre
- Mapear os processos {step=1 .apagado="$step > 1"}
- Escolher as ferramentas {step=2 .apagado="$step > 2"}
- Treinar as equipes {step=3}
Só na última etapa: **tudo pronto!** {if="$step == 3" enter=zoom}
```

A página tem tantas etapas quanto o maior número usado em `step`, em `::: steps` ou numa comparação com `$step`. Para mais, use `--- {steps=N}`. Veja também [Interatividade e navegação](/docs/guides/interactive/#etapas-com-step).

## Como os elementos aparecem

O atributo `enter` escolhe a animação: `fade`, `up` (padrão), `down`, `left`, `right`, `zoom` ou `none`. Ele pode ficar num `@style` para valer em vários elementos: `@style surge {enter=zoom}`.

## Navegando pelas etapas

Quando o documento é mostrado, cada avanço revela a próxima etapa da página atual. Quando todas já apareceram, o próximo avanço vai para a página seguinte. Voltando do começo de uma página, você chega à página anterior já com todas as etapas reveladas.

:::note
Uma visualização estática do documento, como uma imagem ou um PDF, mostra cada página no estado final, com todas as etapas reveladas.
:::

## Erros comuns

| Código | Quando acontece |
| --- | --- |
| R301 | `step` sem valor ou com um valor que não é um número inteiro de 1 a 99, como `{step}`, `step=0` ou `step=a`. |
