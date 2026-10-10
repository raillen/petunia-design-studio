# Workspace

Workspace é composição de interface: janelas, docking, panels, menus e contexto. Não é documento.

Personas customizáveis, temas, iconografia Phosphor/Tabler e feedback seguem [Design visual, personas e iconografia](#/docs/04-ui/visual-design.md). A implementação Qt/QML/CXX-Qt está em `petunia-desktop`; [execução e QA](#/docs/04-ui/native-desktop.md). Gates de tecnologias assistivas e plataformas seguem abertos.

## Qt boundary

Qt/QML e CXX-Qt pertencem exclusivamente à camada de interface.

Outros crates não importam tipos Qt. Core, Engine e Render expõem dados e comandos próprios; `petunia-ui` mantém a sessão headless; `petunia-desktop` converte snapshots e comandos em objetos/props expostos ao QML.

CXX-Qt é a fronteira controlada entre Rust e Qt/C++. Tipos Qt, QObject, sinais/slots e detalhes de janela permanecem nessa borda e não vazam para Core, Engine ou Render.

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

É derivada de active tool + selection capabilities. Evitar regras de domínio espalhadas em QML ou wrappers CXX-Qt; fornecer modelos de actions/controls derivados do estado da aplicação.

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
