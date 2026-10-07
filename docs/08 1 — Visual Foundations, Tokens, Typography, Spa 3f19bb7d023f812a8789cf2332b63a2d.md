# 08.1 — Visual Foundations, Tokens, Typography, Spacing & Motion

# Visual direction

Petunia deve parecer um **instrumento criativo profissional**, não um dashboard web. Superfícies são planas, discretas e densas; separação vem de contraste tonal, hairlines e hierarquia, não de cards excessivos.

# Token layers

1. primitives: palette, spacing scale, radii, font families;
2. semantic: surface.canvas, surface.panel, text.primary, text.muted, border.subtle, action.accent;
3. component: toolbar.height, [dock.tab](http://dock.tab).height, tool.button.size, status.height;
4. state: hover, pressed, focus, disabled, selected, danger, warning.

Tokens vivem em JSON versionado e geram Python constants/QSS/QPalette resources. Feature code não possui cores/metrics literais.

# Color

Dark é default recomendado; light é first-class. UI usa neutral grays com accent Petunia controlado. Canvas selection color pode ser independente do app accent por contraste. Estados críticos nunca dependem apenas de cor.

# Typography

UI usa sans-serif de alta legibilidade, com fallback de sistema. Labels de controles compactos ficam em faixa visual equivalente a 12–13 px em 1x, mas valor real é token + device-independent scaling. Números de precisão usam tabular figures; HUDs de medida usam numeric alignment previsível.

# Spacing

Base 4 px. Densidade Comfortable e Compact aplicam mappings diferentes sem mudar anatomia. Padding mínimo é reduzido em toolbars, maior em dialogs/onboarding.

# Icons

SVG próprio/curado, stroke optical consistency e variantes de estado. IconId é semântico. Toolbar icons não dependem de cor para seleção. Lucide/Tabler podem servir de prior art/bootstrap, mas biblioteca final deve ser Petunia-owned/adequadamente licenciada.

# Motion

100–180 ms para fades/reveals locais; docking drag e tool previews são imediatos. Reduce Motion remove transições não essenciais. Sem bouncing, parallax ou animação decorativa em fluxos de precisão.

# Elevation

Floating palettes/popovers recebem sombra/elevation leve. Docked panels usam border/divider, não cards.

# Focus

Keyboard focus ring é sempre visível quando navegação por teclado está ativa; mouse focus pode ser menos proeminente sem eliminar semântica.

# HiDPI

Todas as medidas são device-independent; canvas render target conhece devicePixelRatio separadamente. Icons devem permanecer crisp em fractional scaling.