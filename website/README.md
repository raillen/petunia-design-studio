# Petunia Design — Architecture Documentation Website

Site estático de documentação da matriz de implementação do Petunia Design Studio.

## Executar localmente

O site carrega Markdown via `fetch()`, portanto deve ser servido por HTTP:

```bash
python3 -m http.server 8080 -d website
```

Depois abra `http://localhost:8080`.

## Estrutura

- `index.html` — shell semântico.
- `styles.css` — layout e estilos acessíveis.
- `app.js` — roteamento, busca, sidebar em acordeão e parser Markdown mínimo.
- `docs/manifest.json` — ordem dos domínios e arquivos.
- `docs/**/*.md` — fonte canônica do conteúdo exibido.

A navegação possui três níveis: **domínio → arquivo → tópico**. Os tópicos são extraídos automaticamente dos títulos `##` de cada Markdown.

## Princípios de leitura

O layout prioriza legibilidade e redução de carga cognitiva: largura limitada, espaçamento amplo, alto contraste, foco visível, atalhos simples, suporte a redução de movimento e hierarquia previsível.