# 08.7 — Design Persona Tool-by-Tool GUI Contract

# Move Tool

**Activation:** V/toolbar. Hover preselect optional; click select; Shift toggles selection; drag moves; Alt/Option duplicates; Shift constrains. Bounding handles resize/rotate. Context bar shows X/Y/W/H, anchor, rotation, lock, transform mode. HUD shows delta/size. Commit = TransformSelectionCommand.

# Node Tool

A ativa nodes para paths selecionados. Click node, marquee nodes, drag handles, Shift multi-select. Node shapes codificam cusp/smooth/symmetric. Context: convert type, join, break, close, reverse, delete-preserve, snapping. Double click segment pode inserir node. Core hit-test/snapping/path mutation em C++.

# Pen Tool

P. Click cria corner; drag cria smooth handles; modifier temporariamente converte/ajusta; click no primeiro node fecha. Preview segue pointer. Context: Pen/Smart/Polygon, fill/stroke, snapping. Esc cancela segmento atual; Enter finaliza path aberto.

# Pencil Tool

Freehand stroke registra batch de pointer samples; stabilizer/smoothing preview; core C++ faz curve fitting. Context: width, smoothing, sculpt/re-edit e pressure.

# Vector Brush

Stroke vetorial com pressure profile/texture quando suportado. Usa input pipeline semelhante ao Pencil, mas cria stroke editável e não pixel dabs.

# Shape Tools

Rectangle, Ellipse, Rounded Rectangle, Polygon, Star, Triangle, Line, Arrow, Cog etc. Drag cria; Shift constrains; Alt center-out. Context expõe parâmetros semânticos. Canvas handles editam parâmetros vivos sem converter para path.

# Corner Tool

Click/drag corners, numeric radius, corner type. Multiple nodes mostram mixed state. Live corner permanece parametric até Bake/Expand.

# Knife/Scissors

Knife drag define cut path; Scissors corta no segment/node. Preview mostra intersections. Topology mutation é undoable.

# Shape Builder

Hover destaca candidate region; click/drag add; modifier subtract. Commit cria compound result e opção keep originals.

# Boolean

Add/Subtract/Intersect/Xor/Divide em toolbar/panel. Live Boolean preserva operands; baked produz geometry result explícito.

# Fill/Gradient

G. Drag gradient axis; handles edit stops/midpoints; context model/type/spread; Color panel edita selected stop. Appearance panel determina qual fill está ativo.

# Transparency

Mesma grammar on-canvas do gradient, mas edita opacity mask/appearance entry.

# Stroke/Profile

Width, caps, joins, dashes, alignment e pressure graph; handles on-canvas quando apropriado.

# Eyedropper

I. Sample display/document/composited/source conforme mode. HUD mostra values/profile; modifier pode copiar appearance.

# Artistic Text

Click cria point text; caret inicia edição. Esc volta selection. Context font/style/size/features.

# Frame Text

Drag cria frame. Overflow indicator, linked-frame ports, columns/insets. IME e shaping completos.

# Text on Path

Path + command/tool; start/end handles, flip, offset, baseline.

# Surface/Artboard

Drag Surface; move/resize existing; presets, dimensions, orientation, bleed, export inclusion.

# Measure

Hover/click/drag mede distance/angle/delta; sem mutation por default.

# Hand/Zoom

Navigation; nunca muta documento.