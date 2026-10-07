# 08.5 — Control Catalog: Fields, Sliders, Pickers, Lists, Trees, Tables & Popovers

# Numeric field

Suporta keyboard entry, drag-to-scrub no label, arrows, Shift fine/coarse policy, units, expressions simples seguras, mixed value, revert, validation inline. Commit on Enter/focus policy é consistente e gera um undo transaction lógico.

# Slider

Track + optional numeric field. Drag preview não cria centenas de history entries; commit final fecha transaction. Alt/Shift modifiers documentados.

# Color control

Swatch button abre color popover/panel, suporta fill/stroke target, none, swap, defaults, eyedropper. Exibe color model/profile quando relevante.

# Combo/dropdown

Type-ahead quando lista grande; recent/favorites opcional; keyboard navigation obrigatória.

# Tree

Layers/Assets usam virtualized model/view, disclosure arrows, drag/drop preview, multi-select, inline rename, badges, visibility/lock toggles com accessible names.

# Table

Data Merge/preflight/jobs usam QTableView model; sortable/filterable quando adequado; não construir milhares de widgets por row.

# Toggle/checkbox

Indeterminate para mixed multi-selection. State icon + accessible text.

# Segmented control

Ações mutuamente exclusivas de tool/context, com arrow navigation.

# Popover

Não modal, anchor-aware, Esc fecha, click outside policy explícita, focus management correto.

# Dialog

Primary action à direita conforme platform conventions; Enter default apenas quando seguro; destructive action styling; no settings giant modal quando painel/preferences page é melhor.

# Tooltips

Nome + shortcut + uma frase de comportamento; delay curto mas não instantâneo. Advanced tooltip pode mostrar modifier hints.