# Política de documentação

A documentação existe para **reduzir ambiguidade de implementação**, não para registrar todo pensamento produzido durante o projeto.

Cada página deve justificar o espaço que ocupa.

## Critério de entrada

Um trecho deve entrar quando pelo menos uma condição for verdadeira:

- define uma decisão de implementação;
- registra uma invariante;
- evita duas interpretações incompatíveis;
- explica algoritmo necessário para implementação correta;
- registra trade-off importante;
- descreve erro ou estado inválido relevante;
- define fronteira entre módulos;
- estabelece critério verificável de conclusão.

Se não satisfaz nenhum desses pontos, provavelmente não pertence à documentação técnica principal.

## Fonte única

Não duplicar uma explicação extensa em várias páginas.

Quando um conceito já possui documento canônico:

> resuma o contexto necessário e crie um link.

Exemplo: `Scene` pode dizer que mutations usam Transactions, mas a semântica completa de Transaction pertence a `commands-history.md`.

## Ordem de leitura

Conteúdo técnico complexo usa divulgação progressiva:

```text
Resumo simples
    ↓
Decisão Petunia
    ↓
Modelo/API
    ↓
Como funciona
    ↓
Matemática/algoritmo
    ↓
Invariantes e limites
```

O leitor deve conseguir entender **o que decidimos** antes de precisar compreender a matemática completa.

## Termos técnicos

Termos inevitáveis são explicados na primeira ocorrência.

Exemplo:

> **Premultiplied alpha** é uma representação em que os canais RGB já estão multiplicados pelo alpha. Ela simplifica composição e reduz artefatos em bordas transparentes.

Depois da explicação inicial, o termo pode ser utilizado normalmente.

Não explicar conceitos básicos repetidamente em todas as páginas.

## Algoritmos

Algoritmos centrais devem documentar:

- objetivo;
- entradas;
- saída;
- etapas;
- invariantes;
- casos degenerados;
- tolerâncias;
- complexidade quando relevante;
- estratégia de erro;
- relação com caches/threads;
- pseudocódigo ou Rust quando melhora entendimento.

Não incluir fórmula apenas para parecer rigoroso.

Quando matemática for necessária, cada símbolo deve ser explicado.

## Código de exemplo

Exemplos são **contratos de direção**, salvo quando marcados como API já implementada.

A documentação deve distinguir:

- implementação atual;
- modelo recomendado;
- decisão fechada;
- questão aberta.

Isso evita confundir proposta com realidade.

## Recursos visuais

Imagem, diagrama ou infográfico entra quando reduz esforço cognitivo em relação ao texto.

Bons candidatos:

- fluxo Core → Engine → Render → UI;
- scene graph;
- cadeia não destrutiva;
- sistemas de coordenadas;
- Bézier e handles;
- premultiplied alpha;
- snapping candidates;
- render graph.

Um único diagrama claro é melhor que vários decorativos.

## Neurodivergência

Para reduzir carga cognitiva:

- parágrafos curtos;
- uma ideia principal por seção;
- títulos descritivos;
- tabelas apenas quando ajudam comparação;
- listas curtas;
- exemplos concretos;
- termos consistentes;
- evitar jargão sem definição;
- evitar alertas visuais excessivos.

Não simplificar conceitos a ponto de torná-los incorretos.

## Living documentation

Quando código altera uma decisão documentada, a documentação muda no mesmo trabalho.

Uma tarefa não está completa quando a documentação canônica conhecida está incorreta.

## Questões abertas

Decisões não fechadas devem ser explicitamente marcadas como abertas.

Não escrever uma preferência ainda em investigação como se já fosse regra constitucional.

## Revisão editorial

Antes de publicar uma seção, perguntar:

1. Isto é necessário para implementar ou revisar?
2. Já está explicado em outro lugar?
3. O texto pode ser menor sem perder precisão?
4. Há termo difícil não explicado?
5. O exemplo acrescenta algo?
6. A matemática é realmente necessária?
7. Está claro o que é decisão e o que é hipótese?
8. Um leitor consegue encontrar a regra principal rapidamente?

Se a resposta for “não”, reescrever antes de adicionar.
