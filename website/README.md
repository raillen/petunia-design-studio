# Petunia Design — Documentation Website

Site estático da matriz arquitetural e de implementação do Petunia Design Studio.

A filosofia é deliberadamente **zero build**: nenhum Node.js, bundler, package manager, node_modules, etapa de geração ou framework SPA é necessário.

## Stack

Tudo é carregado diretamente por CDN:

| Tecnologia | Versão | Papel |
|---|---:|---|
| HTML5 | nativo | estrutura semântica |
| CSS | nativo | tema Petunia inspirado nos princípios do Primer |
| Web Awesome | 3.14.0 | Web Components, native styles e utilidades |
| Phosphor Icons Web | 2.1.2 | iconografia |
| Alpine.js | 3.17.4 | estado pequeno de interface/mobile |
| Marked | 18.0.14 | Markdown → HTML |
| Fuse.js | 7.5.0 | busca fuzzy tolerante a erros |
| Markdown | — | fonte canônica da documentação |

O CSS local não copia o Primer. Ele adota os mesmos **princípios semânticos**: canvas/foreground/border/accent, hierarquia tipográfica discreta, bordas sutis, linhas de leitura curtas, estados claros e responsividade.

## Estrutura

~~~text
website/
├── index.html
├── styles.css
├── app.js
├── README.md
└── docs/
    ├── manifest.json
    ├── about.md
    ├── 00-architecture/
    ├── 01-core/
    ├── 02-engine/
    ├── 03-render/
    └── 04-ui/
~~~

A navegação possui três níveis:

~~~text
Domínio
└── Arquivo
    ├── Visão geral
    └── Tópicos (##)
~~~

Os tópicos são extraídos automaticamente dos headings Markdown.

## Executar localmente

Como o site usa fetch() para carregar os Markdown, sirva a pasta por HTTP:

~~~bash
python3 -m http.server 8080 -d website
~~~

Abra:

~~~text
http://localhost:8080
~~~

## Funcionalidades

- sidebar hierárquica com details nativo;
- busca fuzzy por título, domínio, headings e conteúdo;
- atalho / para focar a busca;
- navegação por teclado nos resultados;
- sumário da página com seção ativa;
- links diretos para headings;
- botão de copiar blocos de código;
- tabelas responsivas;
- navegação mobile;
- light/dark automático pelo sistema;
- prefers-reduced-motion;
- estilos de impressão;
- Markdown GFM via Marked.

## Acessibilidade e neurodivergência

O layout privilegia:

- hierarquia previsível;
- uma área principal de leitura;
- largura de texto controlada;
- contraste forte sem excesso de cor;
- estados de foco visíveis;
- espaçamento regular;
- títulos curtos;
- movimento reduzível;
- pesquisa tolerante a erros;
- elementos nativos sempre que eles já resolvem a interação de forma acessível.

## Política de dependências

As versões dos CDNs são fixadas para manter reprodutibilidade sem package manager.

Se no futuro a documentação precisar funcionar 100% offline, os mesmos assets podem ser copiados para website/vendor/ sem alterar a arquitetura ou introduzir build.

## Segurança do Markdown

O conteúdo Markdown é considerado **conteúdo confiável do próprio repositório**. Marked não é um sanitizador HTML. Se no futuro o site aceitar Markdown de usuários ou fontes externas, adicionar sanitização explícita antes de inserir o HTML no DOM.
