# Cache, overlays e output

Cache acelera; overlay explica interação; output adapta a imagem ao destino.

## O que é cache

**Cache** é uma cópia temporária de um resultado caro de calcular.

Exemplo:

~~~text
path autoral
   ↓
tessellation
   ↓
triângulos
   ↓
cache
~~~

Se o path não mudou, o renderer reutiliza os triângulos em vez de recalculá-los.

Cache é sempre derivado e descartável. Se apagarmos todos os caches, o documento ainda precisa continuar correto.

## Cache layers

Separar caches por responsabilidade:

- evaluated geometry;
- bounds;
- tessellation;
- glyph atlas;
- decoded image;
- raster effect tiles;
- composited group surface.

### Tessellation

**Tessellation** converte formas vetoriais em primitivas que o renderer consegue desenhar eficientemente, normalmente triângulos.

~~~text
curva / fill
     ↓
tessellator
     ↓
triângulos
     ↓
GPU ou software renderer
~~~

Os triângulos não substituem o path original. Eles são apenas uma representação de renderização.

## Invalidation

**Invalidation** significa marcar um cache como desatualizado.

Uma mudança de nome da layer não invalida tessellation. Uma mudança no path sim.

Por isso preferimos revisão e dependências a “limpar tudo”.

~~~text
Path revision 8
   ↓
Tessellation cache revision 8  → válido

Path revision 9
   ↓
Tessellation cache revision 8  → inválido
~~~

## Dependency graph

Um **dependency graph** registra quais resultados dependem de quais entradas.

Se Effect B depende de Effect A, alterar A invalida B. Conteúdo sem dependência de A continua válido.

Isso permite invalidação localizada em vez de reconstrução global.

## Memory budget

Cache acelera, mas usa memória. O renderer precisa de um **memory budget**: limite de memória destinado a caches.

Quando o limite é atingido, resultados menos úteis precisam ser descartados.

## LRU

**LRU** significa *Least Recently Used*.

A ideia é simples:

> quando precisamos liberar espaço, removemos primeiro aquilo que não é usado há mais tempo.

Exemplo:

~~~text
A usado agora
B usado há 2 s
C usado há 20 s
D usado há 3 min
          ↓
remove D primeiro
~~~

LRU é fácil de entender, mas manter uma ordem exata para todo acesso pode ter custo.

## Clock-like eviction

**Clock** é uma aproximação mais barata de LRU.

Cada item possui um marcador de “usado recentemente”. O algoritmo percorre os itens como um ponteiro de relógio:

1. se o item foi usado, limpa o marcador e avança;
2. se já estava sem marcador, ele pode ser removido;
3. continua até liberar memória suficiente.

~~~text
       A*
    ↗      ↘
 D           B
    ↖      ↙
       C*
~~~

`*` representa “usado recentemente”.

A política concreta deve ser escolhida por benchmark; o importante é ter budget e prioridade claros.

## Prioridade de cache

Recursos visíveis e próximos ao viewport têm maior valor imediato.

Uma política pode considerar:

- visibilidade;
- distância do viewport;
- custo de reconstrução;
- tamanho em memória;
- uso recente.

Não transformar essa heurística em estado persistente.

## Overlays

**Overlay** é desenho de interface sobre o canvas que não pertence à arte.

Exemplos:

- selection outline;
- bounding box;
- nodes/handles;
- guides;
- smart guides;
- snap anchors;
- marquee;
- brush cursor;
- measurement labels.

Engine fornece geometria abstrata. Render aplica theme/style.

## Hit feedback

Hover overlay pode atualizar frequentemente sem gerar Command ou nova revisão documental.

Isso é importante porque feedback de ponteiro é estado transitório.

## Output transform

O pipeline final adapta as cores calculadas para o destino:

~~~text
working linear/composite space
→ soft proof opcional
→ tone/gamut mapping quando necessário
→ display transform
→ device surface
~~~

### Tone mapping

**Tone mapping** comprime ou adapta uma faixa de luminância para um destino com capacidade menor.

É especialmente relevante em conteúdo HDR ou quando a tela não representa todo o range calculado.

### Gamut mapping

**Gamut** é o conjunto de cores que um dispositivo ou espaço consegue representar.

**Gamut mapping** adapta cores fora desse conjunto para cores reproduzíveis de acordo com uma política definida.

Essas operações são de visualização/output; não devem alterar silenciosamente as cores autorais do documento.

## Thumbnail

Thumbnail usa o mesmo renderer com overlays desligados e target menor.

Evitar criar um “mini renderer” separado porque duas implementações tenderiam a divergir.

## Headless

**Headless** significa executar sem janela ou interface gráfica.

Export, testes e batch processing precisam funcionar com RenderTarget offscreen sem QApplication.

Isso permite CI, CLI e processamento automatizado usando a mesma lógica de renderização.
