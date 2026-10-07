# Workspace

Workspace é composição de interface: janelas, docking, panels, menus e contexto. Não é documento.

## egui boundary

`egui` pertence exclusivamente à camada de interface.

Outros crates não importam tipos `egui`. O Core, Engine e Render expõem dados e comandos próprios; `petunia-ui` converte esses contratos em widgets e interação.

A escolha de backend de janela/renderização usado para hospedar `egui` é detalhe da camada UI e não deve vazar para os outros domínios.

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

É derivada de active tool + selection capabilities. Evitar regras de domínio espalhadas pelo código de widgets `egui`; fornecer modelos de actions/controls derivados do estado da aplicação.

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
