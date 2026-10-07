# Qualidade de código

Código do Petunia deve ser legível como **prosa técnica precisa**.

A prioridade não é escrever menos caracteres. É reduzir a quantidade de interpretação necessária para entender intenção, invariantes e fluxo.

## Código autoexplicativo

Prefira nomes que respondam “o que isto representa?” sem contexto externo.

Evitar:

```rust
fn upd_pos(p: Pt, d: V2) { ... }
```

Preferir:

```rust
fn update_object_position(
    current_position: DocumentPoint,
    translation: TranslationVector,
) { ... }
```

Abreviações só são aceitáveis quando são convenções universais do domínio e não prejudicam leitura, como `x`, `y`, `rgb` ou `dpi`.

## Newtypes e aliases

Tipos primitivos iguais podem representar conceitos incompatíveis.

Ruim:

```rust
fn move_guide(position: f64, tolerance: f64)
```

Melhor:

```rust
struct DocumentCoordinate(f64);
struct SnapTolerance(f64);

fn move_guide(
    position: DocumentCoordinate,
    tolerance: SnapTolerance,
)
```

Use **newtype** quando o compilador deve impedir a mistura de conceitos.

Use **type alias** quando o objetivo é melhorar leitura, mas os valores continuam semanticamente intercambiáveis.

Exemplo:

```rust
type NodeIndex = usize;
```

Um alias não cria um tipo novo. `NodeIndex` ainda é `usize`.

### Onde newtypes são especialmente úteis

- IDs;
- revisões;
- coordenadas de sistemas diferentes;
- ângulos;
- unidades físicas;
- tolerâncias;
- escalas;
- handles runtime quando não puderem ser confundidos com IDs persistentes.

## Funções

Uma função deve representar uma ação ou cálculo coeso.

Não existe limite mágico de linhas. O sinal de problema é quando a função exige várias explicações independentes.

Extraia funções auxiliares quando isso:

- nomeia uma etapa importante;
- elimina nesting;
- isola validação;
- separa cálculo de efeito colateral;
- permite teste direto;
- reduz carga cognitiva.

Evitar fragmentar uma sequência trivial em dezenas de funções de uma linha sem significado próprio.

## Parâmetros booleanos

Booleanos posicionais escondem intenção.

Evitar:

```rust
render_document(document, true, false);
```

Preferir enum ou options struct:

```rust
struct RenderOptions {
    overlays: OverlayVisibility,
    quality: RenderQuality,
}
```

Boolean é aceitável quando o nome do argumento fica explícito no uso e só existem duas interpretações inequívocas.

Mesmo assim, para API pública ou semântica importante, enums costumam envelhecer melhor.

## Side effects

**Side effect** é qualquer alteração observável fora do valor retornado pela função: escrever em arquivo, alterar documento, emitir evento, atualizar cache global ou modificar estado compartilhado.

Funções de cálculo devem preferir ser puras:

```rust
fn calculate_snap(
    request: &SnapRequest,
    scene: &SceneSnapshot,
) -> SnapResult
```

em vez de:

```rust
fn snap_and_move_selected_object(...)
```

Separar cálculo de mutação torna:

- testes menores;
- undo previsível;
- paralelismo mais seguro;
- erros mais simples;
- comportamento reutilizável por UI, plugins e CLI.

### Side effects permitidos

Efeitos colaterais são necessários em fronteiras como:

- salvar arquivo;
- aplicar Command;
- enviar job;
- publicar evento;
- atualizar cache.

Eles devem estar visíveis pelo nome e pela camada responsável.

## Estado global

Estado global mutável é proibido como mecanismo de conveniência.

Dependências entram explicitamente por:

- parâmetros;
- contexto estreito;
- serviço com responsabilidade definida;
- ownership claro.

Evitar service locators gigantes que apenas escondem globais atrás de métodos.

## Tratamento de erros

Erros de domínio usam tipos específicos.

```rust
enum SceneError {
    ObjectNotFound(ObjectId),
    InvalidParent(ObjectId),
    CycleDetected,
}
```

`thiserror` é apropriado para contratos de Core/Engine.

`anyhow` pode ser usado nas bordas da aplicação para acrescentar contexto, mas não deve apagar erros de domínio que consumidores precisam interpretar.

Evitar `unwrap()` e `expect()` em caminhos recuperáveis, especialmente I/O e entrada externa.

## Unsafe

O código autoral de domínio deve usar:

```rust
#![forbid(unsafe_code)]
```

sempre que a crate permitir.

FFI, quando realmente necessária para integração de plataforma ou bibliotecas nativas, fica confinada a adapters explicitamente auditados. `egui` não deve exigir que o domínio conheça FFI.

Nunca usar `unsafe` para contornar uma dificuldade de ownership antes de revisar o desenho da API.

## Documentação

Comentários não repetem o código.

Ruim:

```rust
// incrementa a revisão
revision += 1;
```

Útil:

```rust
// A revisão muda apenas após commit atômico.
// Previews transitórios não invalidam snapshots persistentes.
revision += 1;
```

Itens públicos importantes devem documentar:

- responsabilidade;
- invariantes;
- unidades/espaço de coordenadas quando relevante;
- erros significativos;
- comportamento não óbvio.

Não é necessário escrever parágrafos para getters evidentes.

## Módulos

Evitar arquivos genéricos como:

```text
utils.rs
helpers.rs
common.rs
misc.rs
```

quando eles se tornam depósitos sem domínio.

Um helper de Bézier pertence a `geometry/bezier.rs`. Conversão de unidade pertence a `units.rs`.

A localização deve ajudar a entender a responsabilidade.

## Revisão

Antes de aprovar código, verificar:

- o nome revela intenção?
- o tipo impede estados inválidos?
- a função possui mais de uma responsabilidade?
- há side effect oculto?
- a camada correta é dona da lógica?
- existe duplicação real?
- o comentário explica algo que o código não consegue dizer?
- a abstração resolve um problema atual?
