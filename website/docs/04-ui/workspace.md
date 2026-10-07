# Workspace

Workspace é composição de interface: janelas, docking, panels, menus e contexto. Não é documento.

## Qt boundary

CXX-Qt, QML/Qt Quick, QtWidgets e biblioteca de docking ficam aqui. Outros crates não importam tipos Qt.

## Shell

```text
Main Window
├── Menu/command surface
├── Toolbar/context bar
├── Dock workspace
│   ├── Layers
│   ├── Inspector
│   ├── Color
│   ├── Stroke
│   ├── Assets
│   ├── History
│   └── ...
└── Document area
    └── Canvas views
```

## Dock state

Posição/tamanho/visibilidade de panels é app/workspace setting. Nunca PTND.

## Panels

Panel é projeção de estado + commands:
- Layers lê scene hierarchy.
- Inspector lê propriedades comuns/selection intersection.
- History lê history service.
- Assets lê registry/library.

Panel não implementa regra de domínio.

## Context bar

É derivada de active tool + selection capabilities. Evitar if/else espalhado em QML; fornecer model de actions/controls.

## Command registry

Cada action deve ter:
- stable ID
- label localizada
- description
- default shortcut
- availability predicate
- execute handler
- category.

Menus, toolbar, command palette e accessibility reutilizam registry.

## Busca

Command palette e busca documental são distintas. No app, busca de actions precisa tolerar aliases e teclado.

## Persistência de workspace

Salvar separadamente:
- geometry de janela
- dock layout
- panel visibility
- toolbar customization
- recent workspaces.

Versionar layout para sobreviver a panels removidos/renomeados.
