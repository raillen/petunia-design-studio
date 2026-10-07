# Composição e efeitos

Composição combina superfícies; efeitos produzem superfícies/intermediários. A ordem é parte da semântica.

## Alpha

Buffers de composição preferencialmente usam premultiplied alpha:

```text
co = cs + cb × (1 - αs)
αo = αs + αb × (1 - αs)
```

onde `cs` e `cb` já estão premultiplicados.

Blend functions como Multiply/Screen operam conceitualmente sobre cores não premultiplicadas; implementação deve respeitar a especificação em vez de simplesmente multiplicar buffers premultiplicados.

## Blend modes

Implementar e testar contra vetores de referência:
Normal, Multiply, Screen, Overlay, Darken, Lighten, Color Dodge/Burn, Hard/Soft Light, Difference, Exclusion, Hue, Saturation, Color e Luminosity.

## Group isolation

Grupo isolado compõe seus filhos em superfície transparente própria e só depois mistura com backdrop. Isso muda o resultado de blend modes e precisa flag explícita.

## Masks

Alpha mask e luminance mask não são equivalentes. O compositor recebe tipo de mask já resolvido e aplica coverage.

## Effects encadeados

```text
object paint
→ shadow
→ blur
→ curves adjustment
→ mask
→ group composite
```

Cada pass declara format, bounds e color-space requirements.

## Blur

Gaussian blur precisa:
- radius/sigma semanticamente definidos
- edge mode
- ROI expansion
- CPU reference
- GPU implementation equivalente.

## Shadows

Shadow é derivada do alpha/shape:
1. extrair coverage
2. offset
3. blur
4. colorize
5. composite na posição correta.

Não deve rasterizar o objeto autoral permanentemente.

## Linear light

Compositing e filtros de luz devem ter política de color space. Não processar indiscriminadamente em sRGB encoded.

## Golden tests

Criar imagens pequenas e determinísticas para alpha edges, isolated groups, blend modes, nested masks e effect ordering.
