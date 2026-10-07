# Sobre

Esta documentação descreve a arquitetura-alvo e as decisões de implementação do **Petunia Design Studio**, editor open source de imagem, vetor e layout.

## Objetivo

Manter quatro fronteiras claras:

- **Núcleo** — dados persistentes e invariantes.
- **Engine** — algoritmos e avaliação.
- **Render** — pixels, composição e visualização.
- **Interface** — interação, Qt/QML, ferramentas e workspace.

## Como usar esta documentação

Cada arquivo corresponde a uma área de implementação. Leia primeiro a decisão, depois invariantes e só então APIs sugeridas. Os exemplos Rust são especificações de direção; não significam que a implementação atual já possua todos os tipos.

## Estado

A documentação é iterativa. Quando uma decisão for consolidada no código, atualizar a página correspondente e, para mudanças estruturais, registrar ADR no repositório.
