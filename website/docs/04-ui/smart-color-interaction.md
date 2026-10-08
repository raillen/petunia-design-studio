# Smart Color — UX de Palette Extract, Recolor Lab, Replace Color e Global Swatches

**Estado:** aprovado por delegação em 2026-10-08; especificação-alvo, **não implementação nem validação empírica**. Respeitar [modelo de Smart Color](#/docs/04-ui/smart-color.md), [Core Color](#/docs/01-core/color.md) e [Color Management](#/docs/02-engine/color-management.md).

## 1. Modelo mental de cor

O usuário precisa distinguir entre (a) escolher uma cor, (b) selecionar objetos que a usam, (c) substituir um valor local, (d) editar um Swatch global, (e) aplicar Live Recolor e (f) materializar o mapeamento. Essas ações são distintas na UI, na semântica do documento e em Undo.

**Fluxo principal:** `Select artwork → Recolor Lab → inspect source colors and swatches → map/restrict → preview → Keep Live / Apply Values / Cancel`.

Em todas as telas mostrar escopo `Selection / Layer / Artboard / Page / Document` e quantidades de objetos/occurrences. `Document` nunca é assumido silenciosamente quando a seleção existe.

## 2. Palette Extraction — descobrir sem aplicar

Selecionar imagem ou vetor → `Extract Palette`. Inspector mostra thumbnail de origem (quando apropriado), perfil/space reconhecido, amostragem/região, `N colors`, agrupamentos ordenados deterministicamente, cobertura estimada e opções `Merge/Split` clusters.

- Extrair é Query/Job cancelável: não cria Swatches automaticamente nem altera imagem.
- Cores de imagem com alpha devem respeitar política explícita de pixels transparentes e background de análise; não considerar RGB invisível como cor visível automaticamente.
- UI mostra cor original e converted/display color, com aviso `Unknown/Missing ICC` quando perfil não está definido; não atribuir conversão arbitrária silenciosamente.
- `Create Swatches` é Command separado, permite nomes e destino `Document palette / Global Swatches`, uma Transaction.
- `Use as Recolor reference` abre Recolor Lab carregando palette proposal e provenance, ainda sem alterar arte.
- Para análise pesada, progresso, cancel, view lazy de imagens, time/memory budget e preview stable. Ordenar por cobertura/distância perceptual com método declarado.

## 3. Recolor Lab — source→target visível

Apresentação preferida: duas colunas simples `Source color` → `New color` com amostra, label semântico, contagem de ocorrências e estado de lock, mais preview grande do artwork ao lado/canvas.

Controles primários: `Palette preset`, `Harmony`, `Shuffle (seed)`, `Preserve lightness`, `Preserve contrast`, `Preview On/Off`, `Keep Live`, `Apply Values`. Avançado: escolha de color space/interpolation, tolerance/perceptual metric, Gamut mapping, Alpha, Scope, Swatch linked, Spot protection.

- **Link source→target:** alterar uma linha não modifica uma cor global sem escolha explícita; mappings repetidos são visíveis.
- **Harmony:** análoga/complementar/triádica/outros presets são propostas de cores, não promessas de contraste ou fidelidade de impressão.
- **Shuffle/Randomize:** seed fixada em spec autoral para efeito live; `Reseed` é ação explícita, não muda a cada novo preview.
- **Preserve lightness / contrast:** constraints aproximadas com validação após color transform e gamut; informar quando não puder satisfazer, oferecendo aceitar compromisso/ajustar manualmente/cancelar.
- **Protect Spot / Global Swatches:** ligado por padrão quando recolor quebraria referências ou separações; desbloqueio explícito, com aviso e listagem de afetados.
- **Keep Live:** cria ColorMappingEffect tipado; `Apply Values` materializa mudanças autorais; `Bake Mapping` remove live effect após confirmação, preservando só result values.
- **Compare:** mostrar before/after no canvas ou alternância rápida. Não selecionar objetos ou reordenar Layers ao alterar preview.

## 4. Recolor from Image — imagem como referência, não vínculo mágico

Escolher ResourceId/imagem → extração assistida → sugerir mapping por relações de luminosidade, hue ou contraste, com preview. Usuario pode travar/editar correspondências manualmente. Apply materializado não cria dependência futura da imagem; Keep Live só guarda reference quando o contrato de efeito suportar isso explicitamente.

Proibir mapping arbitrário se imagem não tem perfil e resultado depende de interpretação de gamut: apresentar `Color interpretation` antes de aplicar. Não confundir `Assign Profile`, `Convert Profile` e `Recolor`.

## 5. Replace Color e Select Similar — verbos diferentes

- `Select Similar Color`: Query → SelectionState, sem alterar DocumentRevision.
- `Replace Color`: filtro `Exact / Perceptual threshold` + scope + preview da contagem → Command/Transaction.
- `Edit Global Swatch`: edição do Swatch vinculado; painel explica que consumidores referenciados mudarão, inclui count e `Detach this occurrence` como opção distinta.
- `Change local color`: edita override específico, não altera Swatch vinculado globalmente sem intenção.
- `Keep Live Mapping`: render deriva cor da regra; Source Paint preservado; Evaluate/Bake explícitos.

Color sources cobrem Fill, Stroke, Gradient Stop, Text Runs/TextStyle, Patterns, Appearance, Symbols e overrides, respeitando propriedade e locking. Preservar transparência quando alterando RGB/CMYK/Lab se o usuário não pediu mudança de alpha.

## 6. Editor de Swatches e harmonias

Swatch card contém chip com contraste de borda, nome, valor com espaço/perfil e `Global / Local / Spot`. Mostrar ícone + texto, não só cor. Link status visível no inspector. Permitir agrupamento por paletas, busca por nome/valor/tags, filtros usados/não usados e auditoria de dependências.

Harmonias ficam em mini painel com resultado previsível: indicar origem e ângulo/param, permitir aceitar como `Swatches`, `Replace mapping` ou cancelar; não aplicar diretamente a dezenas de objetos sem preview.

## 7. Acessibilidade e componentes

- Linhas de mapping acessíveis por teclado, leitor de tela anuncia `source red, target blue, 42 occurrences, protected` (exemplo ilustrativo). Sempre incluir valor textual e space/perfil, além do chip.
- Inputs aceitam formatos suportados por Color Engine, mostram erro de gamut/parse inline; digitação incompleta não escreve dado autoral.
- Sliders de hue/lightness/contrast possuem números e incremento configurável; wheel/picker não são única interface.
- Provide color-blind accessibility: labels, equivalentes numéricos e comparação de luminância/contraste; paleta não é o único canal de significado.
- Com documentos longos, usar busca por source color e lista virtualizada de mappings, com foco estável.
- No Paint/Region Paint, um picker compartilhado de cor pode alimentar current paint, mas não mistura escopos autorais.

## 8. Quality gates

1. `Extract Palette` não suja documento; `Create Swatches` cria uma transação.
2. Keep Live, Apply Values e Bake produzem estados persistentes distintos e reversíveis.
3. Spot/linked swatches não são convertidos/desvinculados sem consentimento.
4. ICC mixed/unknown, alpha, out-of-gamut e gradients não criam alterações silenciosas.
5. Seed deterministic for recolor randomization; preview parity com Commit.
6. Color from image não cria vínculo oculto após Apply.
7. Undo restaura estilos, swatch refs e overrides; snapshot stale impede gravação.
8. Keyboard/screen reader conseguem editar source→target e distinguir status de proteção.
9. Testes de perceptual threshold especificam a métrica (ex.: ΔE apropriado), não distância RGB cartesiana não declarada.

## Referências

[Illustrator — Recolor Artwork](https://helpx.adobe.com/illustrator/using/recolor-artwork.html) · [Figma Draw](https://www.figma.com/blog/introducing-figma-draw/) · [Smart Color model](#/docs/04-ui/smart-color.md)
