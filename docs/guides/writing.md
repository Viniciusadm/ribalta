---
title: Escrevendo documentos
description: Títulos, parágrafos, ênfase, links, listas, citações, código, tabelas, imagens, comentários e notas.
---

Um documento Ribalta é feito de páginas separadas por uma linha com `---`. Veja [Tamanhos e documentos](/guides/pages/).

O conteúdo de uma página é escrito como em Markdown, com uma diferença importante: em uma página, cada linha tem intenção, então uma quebra de linha no texto é uma quebra de linha na página.

## Títulos

Comece a linha com `#` seguido de um espaço. Use de um a seis `#` para os níveis de título:

```ribalta preview
# Título principal
## Subtítulo
### Seção
Texto comum da página.
```

O tamanho, o peso e a fonte de cada nível vêm do tema. O primeiro título da página também dá nome a ela, para o programa que mostra o documento exibir em listas e índices.

## Parágrafos e quebras de linha

Linhas seguidas formam um parágrafo, e cada linha aparece em uma linha própria na página. Uma linha em branco separa um bloco do próximo:

```ribalta preview
# Próxima reunião
Quinta-feira, às 14h
Sala 3, segundo andar

Traga os números do mês anterior.
```

:::note
Diferente do Markdown tradicional, você não precisa terminar a linha com dois espaços para quebrar a linha. Para juntar duas frases na mesma linha da página, escreva-as na mesma linha do texto.
:::

## Negrito, itálico, riscado e código

| Você escreve | Resultado |
| --- | --- |
| `**negrito**` | **negrito** |
| `*itálico*` | *itálico* |
| `~~riscado~~` | ~~riscado~~ |
| `` `código` `` | `código` |

As marcações podem ser combinadas:

```ribalta preview
# Ênfase
Texto em **negrito**, *itálico* e ~~riscado~~.
Código no meio da frase: `npm install`.
**Negrito com *itálico* dentro.**
```

## Links

Escreva o texto entre colchetes e o endereço entre parênteses. São aceitos endereços que começam com `https://` ou `http://` e endereços de e-mail com `mailto:`:

```ribalta preview
# Contato
Conheça o projeto em [nosso site](https://example.com).
Dúvidas: [email@email.com](mailto:email@email.com)
```

## Listas

Comece cada item com `-` (ou `*`, ou `+`). Para listas numeradas, use `1.` ou `1)`. Indente um item para criar uma lista aninhada:

```ribalta preview
# Plano de ação
- Mapear os processos atuais
- Definir as prioridades
  - Curto prazo
  - Longo prazo
- Apresentar à diretoria

---
# Etapas
1. Diagnóstico
2. Proposta
3. Implantação
```

Algumas regras úteis:

- A numeração começa no primeiro número que você escrever: uma lista que começa com `3.` continua em 4, 5 e assim por diante.
- Uma linha indentada logo abaixo de um item, sem marcador, continua o mesmo item em uma nova linha.
- Uma linha em branco termina a lista.

## Citações

Comece as linhas com `>`. Tudo o que estiver depois do `>` é tratado como conteúdo comum, então você pode usar ênfase e várias linhas:

```ribalta preview
> O relatório que levava uma semana agora fica pronto em uma hora.
> — Gerente de operações
```

## Blocos de código

Coloque o código entre duas linhas com três crases. Informe a linguagem depois das crases de abertura para ter realce de sintaxe:

````ribalta preview
# Consulta de vendas
```sql
SELECT nome, total
FROM vendas
WHERE ano = 2026;
```
````

O conteúdo de um bloco de código aparece exatamente como foi escrito. Linhas com `---` dentro dele não começam uma nova página.

## Tabelas

Separe as colunas com `|` e coloque uma linha de traços logo abaixo do cabeçalho. Use `:` na linha de traços para alinhar a coluna:

```ribalta preview
# Planos
| Plano | Usuários | Preço mensal |
| :--- | :---: | ---: |
| Básico | 5 | R$ 49 |
| Equipe | 20 | R$ 149 |
| Empresa | Ilimitado | R$ 399 |
```

| Linha de traços | Alinhamento |
| --- | --- |
| `---` ou `:---` | À esquerda |
| `:---:` | Centralizado |
| `---:` | À direita |

As células aceitam negrito, itálico, código e links. Tabelas também servem de fonte de dados para [gráficos](/guides/charts/).

## Imagens

Use a sintaxe de imagem do Markdown: `!`, a descrição entre colchetes e o endereço entre parênteses. Atributos colados logo depois ajustam o tamanho e o recorte:

```ribalta preview
# Nossa sede
![Paisagem com montanhas](https://images.unsplash.com/photo-1501785888041-af3ef285b470?w=1200){h=600 radius=lg}
```

O endereço precisa começar com `https://` ou ser uma imagem guardada (`asset:`). Veja [Imagens](/guides/images/) para imagens guardadas e `fit=cover`.

:::caution
Deixe uma linha em branco entre um parágrafo e uma imagem. Sem ela, a imagem passa a fazer parte do parágrafo, como uma imagem no meio do texto.
:::

## Comentários

Uma linha que começa com `//` é um comentário: ela não aparece na página nem nas notas. Use para lembretes ou para esconder um trecho temporariamente:

```ribalta preview
// Confirmar os números com o financeiro
# Metas do semestre
Expandir para duas novas regiões.

// Reduzir custos em 10%.
Melhorar o tempo de resposta ao cliente.
```

O `//` só vale no começo da linha. No meio de uma frase, como em um endereço `https://`, ele é texto comum.

## Escapando caracteres

Algumas sequências têm significado especial, como `*`, `#` no começo da linha ou `{` no fim. Para escrever o caractere em si, coloque uma barra invertida `\` antes dele:

```ribalta preview
# Tabela de preços \{2026}

Use \*asteriscos\* sem itálico.

\# Esta linha não é um título.

\- Esta linha não é um item de lista.
```

A barra invertida funciona antes de qualquer sinal de pontuação. Dentro de uma tabela, `\|` escreve uma barra vertical sem criar uma nova coluna.

## Notas

O bloco `::: notes` guarda observações para quem mostra o documento, como o que dizer em cada página de uma apresentação. As notas nunca são desenhadas na página; o programa que mostra o documento decide onde exibi-las, por exemplo numa tela só para quem apresenta:

```ribalta preview
# Resultados do trimestre
A receita cresceu 38%.

::: notes
Comentar que o crescimento veio principalmente de clientes novos.
Abrir para perguntas antes da próxima página.
:::
```

O bloco começa com `::: notes` e termina com uma linha contendo apenas `:::`. O texto dentro dele é guardado como está, sem formatação.

## Próximos passos

- [Layouts](/guides/layout/): organize o conteúdo em colunas, linhas e grades.
- [Estilos](/guides/styling/): mude cores, tamanhos e fontes.
- [Referência da sintaxe](/reference/syntax/): toda a linguagem em uma página.
