# Pendência — Janela de Configurações / Preferências (2026-09-22)

- **Branch:** `refactor/tools-funcionamento-2026-09-22`
- **Status:** PENDENTE (stopgap aplicado, janela real futura)
- **Origem:** Batch 1 da Select pediu "opção no menu de configurações".

## Fato

Não existe tela nem menu de configurações no projeto hoje.

- Zero `struct Preferences/Settings` em `crates/`.
- Zero `Settings/Preferences` funcional em `apps/petunia-design/ui/`.
- `StudioMenuButton` (`ui/app.slint:120-143`) é botão simples, sem dropdown.
- Registry marca como ausente:
  - `ptnd.action.edit.preferences` — `PostV1Candidate, Absent` (`surfaces.rs:130`)
  - `ptnd.window.preferences` — `PostV1Candidate, Absent` (`surfaces.rs:217`)
- Spec existe, código não: `docs/08 12 — Preferences…`, `docs/09 23 — Preferences…`
  (typed `SettingId` registry, `preferences.toml`, generations — nada wired).

## Stopgap aplicado (Batch 1, commit `f717540`)

Regra do marquee (`Sobrepor | Completa | Auto`) ligada como controle
contextual na toolbar (`SECTION 3`), espelho do padrão `snap_toggled`:

- `ui/app.slint`: `marquee_rule` (prop) + `marquee_rule_changed` (callback).
- `src/main.rs`: `on_marquee_rule_changed` → `shell.tools.set_select_marquee_rule()`.
- `sync_ui_from_shell()` espelha o estado da tool na UI.
- Backend pronto: `MarqueeSelectRule::{Intersect, Contained, Directional}`.

Escopo propositalmente local: nada na menu-bar global, nada de dialog.

## O que fazer no futuro (janela real)

1. Criar `Preferences` por `docs/09 23` (camadas, `SettingId` tipado, `preferences.toml`, generations).
2. Tirar `ptnd.action.edit.preferences` e `ptnd.window.preferences` de `Absent`.
3. Migrar para lá: `marquee_rule`, `snap.config`, e futuras opções de tool
   (ex.: cursor contextual da Pen, `PenCursorHint`, pendente de bind na UI).
4. Dar dropdown/popover real ao `StudioMenuButton` ou menu dedicado (hoje é stub).
5. Manter regra de ouro: config de tool é estado de UI/shell, nunca `Command`
   nem `ChangeSet` (precedente: `snap.config` direto em `main.rs:1396-1400`).

## Critério de aceite da pendência

Janela abre via ação `ptnd.action.edit.preferences`, altera e persiste a regra
do marquee, e o controle contextual da toolbar reflete a mesma fonte de verdade.
