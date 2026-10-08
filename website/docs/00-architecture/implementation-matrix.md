# Matriz de implementação

A matriz registra **arquivos existentes**, responsabilidades atuais e módulos necessários para chegar a um editor profissional sem criar god modules.

## Estado atual

| Crate | Arquivo atual | Estado | Próxima decisão estrutural |
|---|---|---|---|
| Core | `color.rs` | RGBA + 3 espaços | separar valor, encoding, perfil e alpha |
| Core | `math.rs` | Point/Vec2/Rect/Transform2D | coordenadas, inversa, bounds robustos, units |
| Core | `path.rs` | nodes cúbicos + contours | IDs internos, segmentos, invariantes e shapes paramétricos |
| Core | `scene.rs` | Path/Group + fill/stroke | hierarquia real, appearance, masks, effects, blend |
| Core | `document.rs` | canvas + scene | pages/artboards, color setup, resources, styles |
| Engine | `command.rs` | Command + undo/redo | transactions, merge/coalescing, preview/commit |
| Engine | `snapping.rs` | guide/grid básico | candidatos, prioridades, screen-space threshold |
| Render Model | nova crate | não existe | contratos imutáveis Engine → Render |
| Render | `backend.rs` | trait mínimo | snapshot + render graph + targets |
| UI | `app.rs` | sessão mínima | tool controllers e estado transitório separado |

## Arquivos-alvo do Núcleo

```text
petunia-core/src/
├── color.rs
├── math.rs
├── units.rs
├── id.rs
├── path.rs
├── shape.rs
├── paint.rs
├── appearance.rs
├── effects.rs
├── scene.rs
├── text.rs
├── raster.rs
├── resources.rs
├── styles.rs
├── symbols.rs
├── guides.rs
├── document.rs
├── serialization.rs
└── error.rs
```

Não é necessário criar todos imediatamente. A regra é extrair quando a responsabilidade deixar de caber claramente no arquivo atual.

## Arquivos-alvo do Engine

```text
petunia-engine/src/
├── command/
├── geometry/
│   ├── bezier.rs
│   ├── bounds.rs
│   ├── intersections.rs
│   ├── boolean.rs
│   ├── offset.rs
│   ├── simplify.rs
│   └── shape_builder.rs
├── spatial/
│   ├── index.rs
│   ├── hit_test.rs
│   ├── snapping.rs
│   ├── alignment.rs
│   └── measurement.rs
├── brush/
├── raster/
├── text/
├── layout/
├── color/
├── import/
├── export/
├── jobs/
└── plugins/
```

## Arquivos-alvo do Render Model

```text
petunia-render-model/src/
├── snapshot.rs
├── primitive.rs
├── paint.rs
├── text.rs
├── image.rs
├── effect.rs
├── composite.rs
└── resource.rs
```

A crate contém apenas DTOs/runtime contracts imutáveis de renderização. Sem backend, cache, Qt ou algoritmos geométricos.

## Arquivos-alvo do Render

```text
petunia-render/src/
├── backend.rs
├── snapshot.rs
├── graph.rs
├── vector/
├── raster/
├── text/
├── compositor/
├── effects/
├── cache/
├── overlay/
├── output/
└── software.rs
```

## Arquivos-alvo da Interface

```text
petunia-ui/src/
├── app.rs
├── session.rs
├── input.rs
├── actions.rs
├── tools/
├── viewport/
├── panels/
├── workspace/
└── accessibility/
```

Qt/QML e CXX-Qt permanecem na borda de UI. Nenhum tipo Qt deve entrar em Core, Engine ou Render.

## Ordem de implementação recomendada

1. Fechar `math`, `path`, `scene` e `document`.
2. Introduzir `Appearance` e modelo não destrutivo de efeitos.
3. Estabilizar Transactions/History.
4. Construir Geometry + Spatial.
5. Criar snapshot de render e compositor correto.
6. Adicionar raster tiles e color management.
7. Expandir text/layout.
8. Só então ampliar Tools e painéis de UI.

A ordem reduz retrabalho porque ferramentas avançadas dependem das invariantes anteriores.
