# Princípios de refatoração

Refatorar no Petunia significa **melhorar estrutura sem perder comportamento correto, precisão ou capacidade de edição**.

Refatoração não é sinônimo de reescrita.

## Objetivo

Toda refatoração deve produzir pelo menos um ganho verificável:

- reduzir acoplamento;
- tornar responsabilidade explícita;
- remover duplicação real;
- melhorar legibilidade;
- eliminar estado inválido;
- simplificar teste;
- reduzir custo mensurável;
- preparar uma capacidade já planejada.

“Código diferente” não é evidência de “código melhor”.

## Regras constitucionais

### Preserve código correto

Código existente deve ser reutilizado quando:

1. produz comportamento correto;
2. respeita as fronteiras atuais;
3. possui nomenclatura compreensível;
4. não introduz custo desnecessário;
5. pode evoluir sem criar dívida estrutural.

Código antigo não é removido apenas por ser antigo.

### Não preserve código incoerente por apego

Reuso não é uma obrigação quando a implementação:

- viola as fronteiras Core/Engine/Render/UI;
- duplica lógica;
- esconde efeitos colaterais;
- depende de estado global;
- mistura modelo persistente com estado de interface;
- exige `unsafe` desnecessário;
- contradiz uma decisão arquitetural fechada.

### Uma responsabilidade por mudança

Refactors devem ter escopo estreito.

Evitar commits que simultaneamente:

```text
trocam arquitetura
+ mudam comportamento
+ redesenham UI
+ alteram formato de arquivo
+ otimizam performance
```

Quando tudo muda ao mesmo tempo, fica difícil provar o que causou uma regressão.

### Arquitetura antes da conveniência da biblioteca

Nenhuma crate externa define a arquitetura do produto.

Tipos de `kurbo`, `egui`, `i_overlay`, `palette` ou qualquer dependência devem atravessar camadas apenas quando essa exposição for deliberada e estável.

Quando a dependência é detalhe de implementação, encapsule-a.

### Sem abstração preventiva

Não criar trait, generic, registry ou sistema de plugins apenas porque “pode ser útil no futuro”.

Abstrações entram quando existe:

- mais de uma implementação real;
- fronteira arquitetural necessária;
- necessidade concreta de teste/substituição;
- API pública que precisa permanecer estável.

## Política de migração

Mudanças estruturais devem preferir migração incremental.

```text
estado atual
   ↓
introduzir nova fronteira
   ↓
migrar um fluxo vertical
   ↓
validar equivalência
   ↓
migrar consumidores
   ↓
remover caminho antigo
```

Evitar manter duas implementações permanentes.

Um caminho antigo pode coexistir temporariamente somente quando houver:

- plano de remoção;
- critério de paridade;
- responsável claro;
- nenhuma ambiguidade sobre qual é o caminho canônico.

## Migração de dados

Mudanças de formato persistente exigem versão e migração explícitas.

Nunca:

- sobrescrever silenciosamente o original durante uma migração arriscada;
- reinterpretar dados antigos sem registrar versão;
- descartar campos desconhecidos quando eles puderem ser preservados com segurança;
- alterar significado de um campo mantendo o mesmo schema.

## Safety tests antes de mudanças profundas

Antes de refatorar código cuja semântica não está completamente óbvia, primeiro capture o comportamento correto em teste.

O teste não deve congelar um bug conhecido.

A sequência recomendada é:

```text
entender
→ caracterizar comportamento correto
→ testar
→ refatorar
→ comparar
```

## Performance

Não trocar clareza por micro-otimização presumida.

Uma otimização estrutural deve responder:

- qual gargalo foi medido?
- com qual cenário?
- qual métrica melhorou?
- qual complexidade adicional foi introduzida?
- o comportamento permaneceu idêntico?

## Critério de conclusão

Uma refatoração termina quando:

- a nova fronteira é a única fonte de verdade;
- o caminho antigo foi removido;
- testes relevantes passam;
- documentação afetada foi atualizada;
- não restaram adapters temporários sem plano;
- performance não regrediu de forma relevante.
