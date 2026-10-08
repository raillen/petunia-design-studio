# Composição e efeitos

Composição combina superfícies; efeitos produzem superfícies intermediárias. A ordem é parte da semântica.

## Alpha

**Alpha** representa quanto um pixel contribui visualmente para o resultado.

- alpha 0 → totalmente transparente;
- alpha 1 → totalmente opaco;
- valores intermediários → transparência parcial.

### Straight alpha e premultiplied alpha

Em **straight alpha**, RGB guarda a cor original e alpha fica separado.

~~~text
vermelho 50% transparente
RGB   = (1.0, 0.0, 0.0)
alpha = 0.5
~~~

Em **premultiplied alpha**, RGB já é multiplicado por alpha:

~~~text
RGB   = (0.5, 0.0, 0.0)
alpha = 0.5
~~~

O Petunia deve preferir premultiplied alpha nos buffers de composição porque ele simplifica a matemática de sobreposição e reduz artefatos em bordas transparentes.

Para a operação “source over”:

~~~text
Cout = Cs + Cb × (1 - αs)
αout = αs + αb × (1 - αs)
~~~

Onde:

- `Cs` — cor da fonte já premultiplicada;
- `Cb` — cor do fundo já premultiplicada;
- `αs` — alpha da fonte;
- `αb` — alpha do fundo;
- `Cout` — cor resultante.

Os blend modes, porém, são definidos sobre relações de cor que não devem ser confundidas com os valores premultiplicados. A implementação precisa converter ou formular corretamente cada modo.

## Blend modes

**Blend mode** define como a cor de uma camada interage com a cor que já existe abaixo dela.

Exemplos:

- Multiply — escurece combinando componentes;
- Screen — clareia;
- Overlay — aumenta contraste usando Multiply ou Screen conforme o fundo;
- Difference — usa diferença entre componentes;
- Hue/Color/Luminosity — combinam componentes perceptuais de duas cores.

Os modos suportados precisam de testes com valores de referência. Não devemos confiar apenas em inspeção visual.

## Group isolation

Um **grupo isolado** primeiro compõe seus filhos em uma superfície transparente própria. Só depois mistura o resultado do grupo com o conteúdo externo.

Sem isolamento, um filho com Multiply pode interagir diretamente com objetos fora do grupo. Com isolamento, ele interage primeiro apenas com os irmãos.

Isso altera o resultado e precisa ser uma propriedade explícita do grupo.

## Masks

Uma **mask** controla onde um conteúdo aparece.

- alpha mask usa o canal alpha da máscara;
- luminance mask usa brilho convertido em cobertura.

Elas não são equivalentes.

O compositor recebe a cobertura já resolvida e aplica essa cobertura ao conteúdo.

## Effects encadeados

Efeitos são processados em ordem:

~~~text
object paint
→ shadow
→ blur
→ curves adjustment
→ mask
→ group composite
~~~

Trocar a ordem pode mudar completamente o resultado.

Cada etapa precisa declarar:

- formato de entrada;
- formato de saída;
- bounds;
- espaço de cor;
- região necessária;
- possibilidade de cache.

## Gaussian blur

**Gaussian blur** suaviza a imagem calculando cada pixel como uma média ponderada dos pixels próximos.

Os vizinhos mais perto recebem peso maior; os mais distantes recebem peso menor segundo uma curva Gaussiana.

O parâmetro `sigma` (`σ`) controla a dispersão dessa curva:

- sigma pequeno → blur estreito;
- sigma grande → blur mais amplo.

### Convolução

A operação de percorrer os vizinhos e aplicar pesos é chamada **convolução**.

Uma Gaussiana 2D pode ser calculada como duas passagens 1D:

~~~text
imagem
 ↓
blur horizontal
 ↓
blur vertical
 ↓
resultado
~~~

Isso é chamado de **filtro separável** e reduz bastante o custo em relação a avaliar uma grande matriz 2D para cada pixel.

### Radius e sigma

A UI pode apresentar radius, mas a implementação precisa definir de forma estável como esse valor vira sigma e até onde o kernel é amostrado.

Essa relação não pode variar entre CPU e GPU.

## Region of Interest — ROI

**ROI** significa *Region of Interest*: a região mínima que precisa ser processada para produzir um determinado resultado.

Um blur solicitado para um retângulo visível precisa ler pixels além desse retângulo, porque pixels vizinhos influenciam a convolução.

~~~text
output necessário
      ↓
expandir pelo alcance do blur
      ↓
input necessário
~~~

Essa expansão evita processar a superfície inteira sem necessidade.

## Shadows

Drop Shadow pode ser construído a partir da cobertura do objeto:

1. obter alpha/coverage;
2. deslocar;
3. aplicar blur;
4. colorir;
5. compor atrás do conteúdo.

O objeto autoral continua intacto.

## Linear light

Valores RGB codificados para tela, como sRGB, não crescem proporcionalmente à quantidade física de luz.

**Linear light** é a representação em que dobrar o valor corresponde aproximadamente a dobrar a intensidade luminosa representada.

Vários filtros e composições produzem resultado mais correto quando executados em linear light.

Por isso o pipeline precisa declarar em qual espaço cada operação acontece, em vez de processar todos os números RGB da mesma maneira.

## Golden tests

**Golden test** compara a saída atual com um resultado de referência previamente aprovado.

Exemplo:

~~~text
cena de teste
   ↓
renderer
   ↓
imagem produzida
   ↓
comparação com imagem de referência
~~~

Eles são úteis para detectar regressões em:

- alpha edges;
- group isolation;
- blend modes;
- nested masks;
- ordem de efeitos.

Golden tests não substituem testes matemáticos de funções individuais; complementam-nos no resultado visual.


## Formato interno do compositor

A v0.1 usa como formato lógico principal:

~~~text
LinearPremultipliedRgba32F
~~~

Isto significa:

- canais RGB em ponto flutuante de 32 bits;
- valores em representação linear do espaço de composição escolhido;
- RGB premultiplicado por alpha;
- extended range permitido durante processamento quando a operação suportar.

Buffers de saída 8/16-bit são conversões de fronteira, não o espaço de cálculo principal.

O compositor não converte todo documento autoral para esse formato permanentemente. A transformação ocorre em dados derivados de render.

## Espaço de composição

Cada RenderFrame resolve um `CompositingColorContext` derivado do documento e destino.

Direção:

~~~text
authorial/process color
↓ Color Management Engine
linear compositing RGB
↓ compositor/effects
output transform
↓ display/export
~~~

CMYK e Spot permanecem autorais no Core. Para preview em tela, uma representação RGB derivada é usada. Export compatível pode preservar separações/spot por pipeline específico sem reescrever o documento.

## Source Over como default

`Normal` usa Porter-Duff Source Over em premultiplied alpha.

A implementação precisa satisfazer propriedades testáveis:

~~~text
source alpha = 0
→ destination unchanged

source alpha = 1
→ source replaces destination under Normal

transparent black
→ neutral
~~~

Não implementar alpha por fórmulas ad hoc em cada primitive.

## Contrato de BlendMode

BlendMode recebe cores **não premultiplicadas** semanticamente quando sua fórmula exige componentes de cor e depois retorna ao fluxo premultiplied.

Pipeline conceitual:

~~~text
premultiplied source/backdrop
↓ safe unpremultiply when alpha > 0
blend function on color components
↓ combine with Porter-Duff coverage
premultiply result
~~~

Alpha zero nunca causa divisão por zero; componente de cor de pixel totalmente transparente é tratado de forma canônica.

Os modos built-in seguem um contrato único compartilhado entre software renderer e futuro backend GPU. Não existirão versões “quase Multiply” por backend.

## Group isolation policy

Grupo precisa de superfície isolada quando pelo menos uma destas condições exigir semântica própria:

- group opacity diferente de 1;
- group blend mode que precisa compor o resultado agregado;
- mask/clip pós-filhos;
- post-paint effect no grupo;
- explicit isolation;
- operação que exige backdrop local.

Grupo simples `Normal + opacity 1 + sem effects/mask` pode ser flattenado no graph sem surface intermediária quando isso preservar exatamente o resultado.

Essa otimização é derivada; não altera Scene.

## Mask representation

Masks resolvidas pelo Render usam coverage float linear:

~~~text
0.0 → sem cobertura
1.0 → cobertura total
~~~

Alpha mask lê alpha.

Luminance mask calcula luminância no espaço/transferência definida pelo contrato antes de multiplicar coverage. Não usar média ingênua de R/G/B.

Coverage de mask multiplica alpha/cobertura da entrada; não altera Source autoral.

## Gaussian Blur contract

O parâmetro autoral canônico de Gaussian Blur é `sigma`, em unidades coerentes com o espaço em que o efeito é definido.

Para execução raster:

~~~text
kernel_extent = ceil(3 × sigma)
~~~

por eixo como política inicial.

Amostras além de ±3σ são truncadas e os pesos restantes são renormalizados. CPU e GPU usam a mesma relação.

Se a UI futura quiser apresentar “radius”, ela converte para sigma explicitamente; radius visual não vira uma segunda semântica do efeito.

### Edge behavior

Gaussian Blur documental usa **transparent black** fora da surface lógica do conteúdo.

Isso permite shadows/glows expandirem naturalmente em surfaces intermediárias transparentes.

Filtros que precisam Clamp/Repeat/Mirror declaram isso separadamente em seu contrato.

## Blur implementation

Para sigma pequeno/médio, usar kernel Gaussiano separável.

~~~text
horizontal convolution
↓
vertical convolution
~~~

Para sigma muito grande, a implementação pode trocar internamente para aproximação equivalente mais eficiente somente se ficar dentro da tolerância visual/documentada do backend de referência.

A escolha otimizada não altera params autorais.

## Drop Shadow

Contrato:

~~~text
source alpha/coverage
↓ offset
↓ Gaussian Blur(sigma)
↓ colorize in compositing space
↓ opacity
↓ Source Over behind original content
~~~

Shadow bounds expandem por offset + kernel extent.

Inner Shadow usa máscara/interseção inversa apropriada e não é implementado simplesmente “com offset negativo”.

## Adjustments

Adjustments declaram qual domínio de cor recebem.

### Levels

Opera por canais em representação definida pelo adjustment, com input black/white, gamma e output range validados.

Gamma precisa ser positivo e finito.

### HSL

Hue/Saturation/Lightness não deve ser implementado convertendo arbitrariamente valores premultiplied. Primeiro trabalha sobre componentes de cor não premultiplicados no espaço especificado.

### Curves

Curve adjustment usa função monotônica/lookup derivada dos control points quando a curva precisar preservar monotonicidade. A representação autoral guarda control points; LUT é cache.

Cada adjustment documenta se trabalha em encoded, linear ou espaço perceptual. Não assumir um único espaço para todos.

## Effect tiles

Efeitos raster são avaliados por tiles/ROI.

~~~text
output tile
↓ effect input_region()
required input tiles
↓ evaluate
output tile
~~~

Cache key inclui:

- input revision/key;
- EffectId/parameter revision;
- tile coordinate;
- quality;
- compositing color context;
- backend semantic version quando necessário.

## Numeric behavior

Buffers float podem conter valores RGB fora de 0…1 durante efeitos.

Alpha/cobertura deve permanecer em range semântico válido salvo operação explicitamente HDR/coverage-specialized.

NaN/Inf gerado por efeito é erro de backend/algoritmo: não é propagado silenciosamente ao frame.

## Golden + analytic tests

Blend modes e Porter-Duff possuem testes numéricos com valores conhecidos.

Effects complexos possuem golden tests.

Casos mínimos:

- transparent source/backdrop;
- alpha 0/1;
- semitransparência;
- nested isolated groups;
- alpha e luminance masks;
- blur na borda da surface;
- shadow com offset positivo/negativo;
- extended-range RGB;
- CPU reference repeatability.

## Invariantes do compositor

1. Formato lógico principal é Linear Premultiplied RGBA f32.
2. Conversão de cor para composição é derivada; não altera Core.
3. Normal usa Porter-Duff Source Over.
4. Blend formulas possuem contrato único entre backends.
5. Group isolation é semântica explícita e pode ser otimizada apenas quando equivalente.
6. Masks trabalham como coverage derivada.
7. Gaussian Blur persiste sigma; kernel inicial usa ±3σ renormalizado.
8. Fora da surface, blur documental lê transparent black.
9. Effects declaram color-space semantics e ROI.
10. NaN/Inf não é output válido do compositor.
11. CPU renderer é referência de comportamento.
