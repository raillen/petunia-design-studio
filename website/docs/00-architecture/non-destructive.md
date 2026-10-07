# Não destrutibilidade

Não destrutibilidade é uma **regra de arquitetura**, não apenas uma opção de filtro. O documento deve preservar a fonte e representar a edição como parâmetros reavaliáveis.

## Modelo mental

```text
Source
  ↓
Geometry operations
  ↓
Paint / Appearance
  ↓
Effect stack
  ↓
Mask / Clip
  ↓
Blend / Composite
  ↓
Output
```

Cada etapa pode ser ativada, desativada, reordenada ou alterada quando semanticamente permitido.

## Stack versus grafo

Para um objeto simples, uma lista ordenada é suficiente:

```rust
struct EffectStack {
    items: Vec<EffectInstance>,
}
```

Porém algumas operações possuem múltiplas entradas: Live Boolean, blend entre objetos, máscaras compostas e grupos de warp. Portanto o modelo de avaliação deve aceitar uma **DAG** — Directed Acyclic Graph — mesmo que a UI inicialmente mostre uma stack.

```rust
struct EffectInstance {
    id: EffectId,
    enabled: bool,
    opacity: f32,
    blend_mode: BlendMode,
    mask: Option<MaskRef>,
    kind: EffectKind,
}
```

`EffectKind` é um enum serializável com parâmetros tipados. Evitar “nome + HashMap<String, Value>” no formato nativo porque isso perde validação e dificulta migração.

## Fases de avaliação

Nem todo efeito ocorre no mesmo ponto.

| Fase | Exemplos | Resultado |
|---|---|---|
| Geometry | contour, corner, warp, live boolean | geometria vetorial |
| Paint | fill, stroke, variable width | primitives de pintura |
| Image effect | blur, shadow, glow | superfície/intermediário |
| Adjustment | curves, HSL, levels | transformação de cor |
| Composite | opacity, blend, mask | composição final |

A ordem entre fases deve ser explícita. Um Gaussian Blur não deve acidentalmente modificar os pontos de um path; um Offset Path não deve operar sobre pixels já rasterizados.

## Bake explícito

Operações destrutivas precisam de nomes explícitos:

- **Expand Stroke**: stroke → path.
- **Expand Appearance**: efeitos vetoriais → geometria resultante.
- **Rasterize**: conteúdo → pixels.
- **Bake Filter**: filtro live → pixels modificados.
- **Flatten**: composição de várias camadas → raster.

O usuário deve conseguir distinguir “adicionar live blur” de “aplicar blur destrutivo”.

## Revisões e invalidação

Cada objeto e operação precisa participar de revision tracking.

```text
cache key =
node_id
+ node_revision
+ upstream_revision
+ render_scale
+ working_color_space
+ backend_features
```

Ao alterar apenas a cor do fill, o sistema não deve recalcular uma operação geométrica cara anterior. Ao alterar a geometria, bounds, tesselação, snap index e efeitos dependentes precisam ser invalidados.

## Region of Interest

Filtros raster devem declarar quanto expandem a região necessária. Blur, shadow e glow precisam de pixels além do bounds original. O evaluator consulta o efeito de trás para frente para descobrir a área de entrada necessária e evita processar a tela inteira.

## Preview sem poluir undo

Durante arraste de slider:

```text
pointer down → begin transaction
pointer move → transient parameter override
render preview
pointer move → replace override
pointer up   → commit 1 command
```

O histórico recebe uma única ação, não centenas.

## Ciclos

Máscaras, symbols, clones e effect graphs podem criar referências. O Core deve rejeitar ciclos proibidos no momento da mutação. A avaliação nunca deve “resolver” ciclo por timeout.

## Contrato mínimo de um efeito

Todo efeito deve declarar: versão do schema; parâmetros; fase; tipo de entrada; tipo de saída; bounds expansion; suporte CPU/GPU; determinismo; política de cache; comportamento em color space linear; serialização; e estratégia de migração.

## Referência de produto

Editores não destrutivos modernos permitem reajustar, reordenar e remover operações sem alterar permanentemente a fonte. A arquitetura do Petunia deve garantir isso no modelo de dados, não reproduzir apenas a aparência da UI.
