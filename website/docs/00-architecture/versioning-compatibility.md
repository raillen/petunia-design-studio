# Versionamento e compatibilidade

O Petunia possui vários contratos versionados com ciclos de vida diferentes.

> **Versão do aplicativo não é versão do documento, do plugin nem da semântica de um algoritmo.**

Misturar essas versões tornaria migrations imprevisíveis.

## Camadas de versão

~~~text
ApplicationVersion
ContainerVersion
SchemaVersion
CapabilityVersion
OperationSemanticVersion
PluginPackageVersion
HostApiVersion
PluginDataSchemaVersion
RecoverySchemaVersion
FragmentSchemaVersion
~~~

Cada uma responde a uma pergunta diferente.

## ApplicationVersion

É a versão do produto/binário.

Exemplo:

~~~text
0.1.0
0.2.0
1.0.0
~~~

Ela pode seguir Semantic Versioning quando o projeto estabilizar sua política pública.

ApplicationVersion serve para diagnóstico/release.

Não determina automaticamente se um PTND precisa de migration.

## ContainerVersion

`container_version` identifica a estrutura física do pacote PTND.

Na v0.1:

~~~text
container_version = 1
→ ZIP/ZIP64
~~~

Só incrementa quando a organização física/decodificação básica deixa de ser compatível com readers anteriores.

Trocar nível de DEFLATE ou ordenar entries de forma diferente não exige nova ContainerVersion se a semântica física continua compatível.

## SchemaVersion

`schema_version` identifica o formato lógico de `document.json`/DTO.

Exemplo:

~~~text
schema 1
↓ migration
schema 2
~~~

Incrementar quando:

- campo persistente muda semântica;
- estrutura obrigatória muda;
- novo tipo não pode ser representado no schema antigo;
- invariantes persistentes exigem migração.

Não incrementar só porque uma struct Rust foi renomeada internamente.

## minimum_reader_version

Manifest pode declarar `minimum_reader_version` para rejeição rápida por versões muito antigas do aplicativo.

Ela é metadata auxiliar.

As autoridades reais de compatibilidade continuam sendo:

~~~text
ContainerVersion
+
SchemaVersion
+
required_capabilities
~~~

Não usar string de versão do app como substituto dessas três verificações.

## Required Capability

Capability identifica uma feature persistente que precisa ser entendida para fidelidade correta.

Formato conceitual:

~~~text
core.parametric-shape.v1
core.spot-color.v1
plugin:com.example.effect@2
~~~

A versão faz parte da identidade do contrato.

Se uma capability obrigatória for desconhecida:

~~~text
não abrir como se estivesse tudo correto
~~~

O reader pode:

- rejeitar;
- abrir read-only/degraded quando conseguir provar segurança;
- preservar payload sem permitir edição destrutiva.

A política visual vem depois; o motor nunca ignora silenciosamente.

## Optional extension

Payload opcional namespaced pode ser preservado mesmo quando não compreendido.

~~~text
known?    → parse + validate
unknown?  → preserve opaque
required? → incompatibility if unsupported
~~~

Preservar não significa executar.

## OperationSemanticVersion

Algumas operações persistem apenas parâmetros, enquanto o algoritmo que os interpreta vive no Engine/Render.

Se mudar o algoritmo puder mudar visivelmente o mesmo documento:

~~~text
same params
old engine → result A
new engine → result B
~~~

então existe mudança de **semântica**, não simples otimização.

Exemplos:

- Brightness/Contrast formula;
- Image Trace corner mapping;
- QR encoding policy quando existe escolha não determinada pelo standard;
- blur edge policy;
- gradient midpoint mapping;
- curve fitting policy quando resultado materializado depende dela.

Soluções permitidas:

1. manter implementação antiga para semantic version antiga;
2. migrar parâmetros para preservar resultado;
3. armazenar OperationSemanticVersion quando necessário.

Nunca mudar aparência de documento antigo silenciosamente sob o rótulo de refactor.

## Quando não versionar algoritmo

Algoritmo interno não precisa de versão persistente quando diferentes implementações obedecem ao mesmo contrato dentro da tolerância definida.

Exemplo:

~~~text
CPU blur implementation A
GPU blur implementation B
↓
mesma semântica + tolerância visual
~~~

Isso é backend optimization, não nova semântica autoral.

## Render Model

`petunia-render-model` é contrato interno runtime.

Ele não possui migration PTND.

Mudanças seguem o código do workspace e precisam apenas manter Engine/Render compilando sob a mesma versão.

Nunca serializar RenderSnapshot como documento persistente para “evitar migration”.

## PluginPackageVersion

É a versão do pacote de um plugin.

Pode mudar por:

- bugfix;
- novos entrypoints;
- performance;
- metadata.

Não implica mudança de Host API.

## HostApiVersion

Host API define o protocolo público Petunia ↔ plugin.

Compatibilidade precisa ser negociada antes de instanciar o plugin.

Direção:

~~~text
plugin requires Host API 1.x
host supports 1.y
→ compatible according to contract

plugin requires Host API 2
host supports 1
→ do not load
~~~

O esquema concreto pode usar major/minor ou capability set; o princípio é negociação explícita.

## PluginDataSchemaVersion

Payload persistente de plugin possui versão própria.

~~~text
plugin package 5.3
payload schema 2
host API 1
~~~

Essas versões podem coexistir.

Atualizar plugin não deve reinterpretar payload antigo sem migration do próprio plugin/host contract.

Se plugin não estiver disponível, payload permanece opaco quando a capability permitir.

## RecoverySchemaVersion

Recovery/journal é runtime persistente, mas não PTND autoral.

Ele possui versão própria porque:

- Transaction DTO pode evoluir;
- checkpoint layout pode mudar;
- records precisam ser validados antes de replay.

Upgrade da aplicação pode migrar ou descartar recovery incompatível com diagnóstico; nunca misturar records de versões sem verificar.

## FragmentSchemaVersion

`DocumentFragment` possui schema separado.

Isso permite copy/paste entre processos/versões sem exigir que o clipboard contenha um PTND completo.

Compatibilidade segue:

~~~text
fragment old migrável
→ migrate
→ validate
→ remap IDs
→ paste
~~~

Fragmento futuro incompatível não é inserido parcialmente.

## Migration direction

Migrations PTND oficiais são **forward**:

~~~text
old schema
↓
current schema
~~~

Save normal escreve o schema atual.

A v0.1 não promete “Save As PTND v1 antigo” depois que existir v2.

Export para formatos externos é outra operação, não downgrade do documento nativo.

## Migration purity

Migration precisa ser:

- determinística;
- sem UI;
- sem network;
- sem depender de filesystem externo salvo indicação explícita;
- testável por fixture;
- idempotente no sentido de não aplicar a mesma etapa duas vezes.

A migration transforma DTO → DTO.

Só depois o Core constrói Document.

## Migration failure

Se migration falhar:

- arquivo original não é alterado;
- nenhum save automático sobrescreve a source;
- diagnóstico inclui versão/etapa;
- recovery/cópia temporária pode ser usada para investigação.

Nunca “tentar consertar” destruindo campos desconhecidos e salvar por cima.

## Compatibility matrix

Reader mantém uma matriz explícita:

~~~text
ContainerVersion supported?
SchemaVersion supported/migratable?
RequiredCapabilities supported?
Plugin required payload supported?
↓
OpenMode
~~~

OpenMode pode ser:

~~~text
Editable
ReadOnlyDegraded
Unsupported
~~~

A semântica de UI desses estados será discutida depois.

O motor precisa apenas produzir o diagnóstico/capabilities resultantes.

## Forward compatibility

Schema v1 usa `additionalProperties: false` nos objetos built-in conhecidos.

Nova estrutura built-in incompatível exige SchemaVersion nova em vez de depender de readers antigos ignorarem campos essenciais.

Extensões opcionais utilizam namespace/payload próprio justamente para permitir preservação sem expandir o schema core.

## Version pinning em caches

Caches derivados podem incluir semantic version na key quando o algoritmo relevante mudou.

~~~text
input identity
+ params
+ algorithm semantic version
→ cache key
~~~

Atualizar algoritmo não pode reutilizar cache produzido por semântica anterior.

## Tests

Cada versão persistente relevante possui fixtures.

PTND:

~~~text
schema N fixture
↓ migration
current DTO
↓ save
↓ reopen
same semantic document
~~~

Effects/operations:

~~~text
params + semantic version
↓ reference output
~~~

Plugins:

~~~text
Host API compatibility table
PluginDataSchema migrations
~~~

Recovery/Fragments:

~~~text
old fixture
↓ migrate/reject deterministically
~~~

## Deprecation

Antes de 1.0, contratos ainda podem mudar mais rápido, mas mudança continua exigindo migration/documentação quando já existe fixture persistente.

Após 1.0, remoção de schema/Host API suportado exige política explícita de suporte, não exclusão casual.

## Invariantes

1. ApplicationVersion nunca substitui SchemaVersion.
2. ContainerVersion e SchemaVersion são independentes.
3. RequiredCapability faz parte da compatibilidade autoral.
4. Mudança visual/semântica de operação persistente exige versionamento ou migration.
5. Otimização dentro do mesmo contrato não cria nova semântica.
6. Render Model não é formato persistente.
7. Plugin package, Host API e plugin data schema são versões independentes.
8. Recovery e DocumentFragment possuem schemas próprios.
9. Migrations oficiais são forward e determinísticas.
10. Migration falha sem alterar o arquivo original.
11. Reader calcula compatibilidade antes de expor Document editável.
12. Cache não reutiliza resultado de semantic version incompatível.
