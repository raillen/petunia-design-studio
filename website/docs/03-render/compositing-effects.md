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
