# Manifesto de engenharia

Este documento é o **Norte técnico e humano** do Petunia Design Studio. Ele existe para resolver uma pergunta simples quando duas soluções parecem aceitáveis:

> **Qual delas preserva melhor a simplicidade, a precisão, a liberdade criativa e a capacidade de evolução do Petunia?**

O manifesto define princípios. Detalhes de implementação pertencem às páginas específicas de Core, Engine, Render e Interface.

## Objetivo

O Petunia Design Studio é um editor open source de imagem, vetor e layout que busca oferecer poder profissional sem aceitar como inevitáveis o peso, a complexidade e o acoplamento acumulados por aplicações antigas.

O projeto prioriza:

- inicialização rápida;
- uso responsável de CPU e memória;
- alta precisão geométrica;
- edição não destrutiva;
- arquitetura modular;
- código legível;
- acessibilidade;
- segurança por padrão;
- dependências pequenas e justificadas.

Performance não significa sacrificar clareza. Minimalismo não significa reimplementar bibliotecas maduras sem necessidade.

## Performance minimalista

Hardware do usuário é um recurso que deve ser respeitado.

A regra não é “usar o mínimo de crates possível”. A regra é:

> **cada dependência e cada alocação devem justificar o custo que introduzem.**

O Core, Engine e Render autorais são escritos em Rust. Bibliotecas especializadas são utilizadas quando resolvem problemas difíceis melhor do que uma implementação própria razoável.

Exemplos da stack atual:

| Área | Tecnologia |
|---|---|
| Geometria Bézier e álgebra | `kurbo` |
| Operações booleanas | `i_overlay` |
| Matemática de cor | `palette` |
| Text shaping | `rustybuzz` |
| Rasterização de glifos | `fontdue` |
| Imagens | `image` |
| Paralelismo CPU | `rayon` |
| Persistência | `serde` |
| Identidade | `uuid` |
| UI | `egui` |

O Petunia mede antes de otimizar. Uma abstração clara só deve ser substituída por uma implementação mais complexa quando profiling demonstrar necessidade.

## Soberania do domínio

O documento e suas regras não pertencem à interface, ao renderer nem a uma biblioteca externa.

A direção conceitual é:

```text
Interface
   ↓ intenção
Engine
   ↓ operação
Core
   ↓ snapshot
Render
   ↓ pixels
Interface
```

A UI é uma camada inteligente de interação, mas não autoridade sobre o domínio.

`egui` pode conhecer widgets, painéis, foco e eventos. `egui` não decide geometria, snapping, topologia, regras de documento ou comportamento de efeitos.

## Paradigma não destrutivo

O trabalho original do artista é preservado sempre que a operação puder ser representada como intenção editável.

```text
Fonte
  ↓
Operações geométricas
  ↓
Appearance
  ↓
Efeitos
  ↓
Máscaras e recortes
  ↓
Composição
  ↓
Resultado
```

Operações como boolean, blur, contour, máscaras e ajustes devem permanecer editáveis quando tecnicamente viável.

Transformações destrutivas continuam existindo, mas precisam ser explícitas: **Expand**, **Rasterize**, **Bake** e **Flatten** são ações diferentes e nunca devem ocorrer silenciosamente.

## Clareza cognitiva

Complexidade interna não deve ser transferida ao usuário nem ao desenvolvedor.

Para o usuário:

- hierarquia visual previsível;
- navegação por teclado;
- estados claros;
- movimento reduzível;
- alto contraste;
- termos consistentes;
- ferramentas complexas com apresentação progressiva.

Para o desenvolvedor:

- nomes completos e precisos;
- funções coesas;
- tipos que expressem intenção;
- erros como dados;
- documentação objetiva;
- ausência de estado global oculto;
- decisões arquiteturais explícitas.

## Segurança por padrão

O código autoral de domínio deve evitar `unsafe`.

Quando FFI for inevitável por integração de plataforma ou biblioteca nativa, ela permanece confinada e auditável. A UI em `egui` não muda essa regra.

Entradas externas são consideradas não confiáveis:

- documentos;
- imagens;
- fontes;
- plugins;
- arquivos importados;
- conteúdo compactado.

Falhas recuperáveis retornam erros tipados. Corrupção silenciosa de documento é inaceitável.

## Regra final

Quando houver conflito entre conveniência imediata e integridade arquitetural:

> **prefira a solução que deixa o sistema mais previsível daqui a cinco anos, desde que o custo atual seja proporcional ao problema real.**
