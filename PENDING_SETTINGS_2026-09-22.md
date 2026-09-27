# Pendência — Janela de Configurações / Preferências (2026-09-22)

- **Branch:** `refactor/tools-funcionamento-2026-09-22`
- **Status:** ACESSO GLOBAL IMPLEMENTADO NA WORKTREE FREYA (persistência global ainda não conectada)
- **Origem:** Batch 1 da Select pediu "opção no menu de configurações".

## Fato

A pendência original foi registrada contra o shell Slint histórico; a worktree Freya já tem um diálogo de personalização funcional.

- Não existe `struct Preferences/Settings` em `crates/`.
- A entrada de configuração Freya é a engrenagem da context toolbar.
- `StudioMenuButton` (`ui/app.slint:120-143`) permanece apenas como histórico do shell anterior.
- Registry marca a ação de edição como ligada:
  - `ptnd.action.edit.preferences` — `PostV1Candidate, Wired` (`surfaces.rs:130`)
  - `ptnd.window.preferences` — `PostV1Candidate, Absent` (`surfaces.rs:217`)
- Spec existe, código não: `docs/08 12 — Preferences…`, `docs/09 23 — Preferences…`
  (typed `SettingId` registry, `preferences.toml`, generations — nada wired).

## Implementado na integração Freya

- `CustomizeDialog` agora edita a barra de contexto e o `ToolRailState` da persona atual.
- O rail permite ocultar/mostrar grupos, adicionar/remover ferramentas, reordenar membros e grupos, mesclar um grupo com o anterior e mover uma ferramenta para outro grupo.
- A preferência de uma ou duas colunas é responsiva: a barra volta para uma coluna quando a altura disponível é curta.
- O botão de personalização continua acessível pela engrenagem da context toolbar; `Edit > Preferences` e `Ctrl+,` abrem o mesmo diálogo pela mesma sessão.

A persistência em `preferences.toml`, a janela `ptnd.window.preferences` e as demais camadas de preferências continuam fora desta fatia; o estado de layout é de sessão da UI.

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
2. Tirar `ptnd.window.preferences` de `Absent` quando a janela real existir.
3. Migrar para lá: `marquee_rule`, `snap.config`, e futuras opções de tool
   (ex.: cursor contextual da Pen, `PenCursorHint`, pendente de bind na UI).
4. Preservar a entrada global já exposta e substituir a ação de sessão quando a janela real de Preferences existir.
5. Manter regra de ouro: config de tool é estado de UI/shell, nunca `Command`
   nem `ChangeSet` (precedente: `snap.config` direto em `main.rs:1396-1400`).

## Critério de aceite da pendência

A engrenagem, `Edit > Preferences` e `Ctrl+,` abrem a configuração Freya; a barra de contexto e o ToolRailState refletem a mesma sessão, e a persistência global continua como evolução separada.
