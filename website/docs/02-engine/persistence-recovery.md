# Persistência, Autosave e Recovery

Persistência confiável precisa proteger o trabalho do usuário contra crash, falta de energia, erro de disco e falha durante save.

> O PTND principal representa um save confirmado. Recovery é infraestrutura separada e nunca substitui silenciosamente o arquivo principal.

## Três estados diferentes

~~~text
Saved PTND
→ último estado confirmado pelo usuário

Working Document
→ estado autoral atual em memória

Recovery State
→ material suficiente para recuperar trabalho não salvo
~~~

Eles não são a mesma coisa.

## Save manual

Save trabalha sobre snapshot consistente:

~~~text
current DocumentRevision
↓ immutable snapshot
DTO
↓ PTND temp
flush + validate
↓ atomic replace
saved_revision = current_revision
~~~

O save não bloqueia o Document por toda a serialização quando snapshot suficiente já foi capturado.

Se o usuário continuar editando enquanto o save roda:

~~~text
save snapshot revision 42
Document avança para revision 43
↓
save 42 termina
saved_revision = 42
current_revision = 43
→ documento continua dirty
~~~

Nunca marcar revision 43 como salva só porque um job iniciado em 42 terminou.

## Save result e revision

Resultado de save carrega a revision persistida.

Direção:

~~~rust
pub struct SaveResult {
    pub document: DocumentId,
    pub revision: DocumentRevision,
    pub destination: DocumentLocation,
}
~~~

A aplicação atualiza o checkpoint somente se o resultado ainda pertence ao documento aberto correspondente.

## Autosave

Autosave não sobrescreve o PTND principal por padrão.

Ele grava estado de recuperação em storage separado.

Razões:

- save principal pode ser destino remoto/lento;
- recovery precisa funcionar mesmo quando o documento nunca foi salvo;
- não queremos substituir um arquivo confirmado por uma edição transitória sem ação explícita;
- autosave pode usar formato operacional otimizado diferente.

## Recovery store

Direção conceitual:

~~~text
RecoveryRoot/
└── <DocumentId>/
    ├── metadata.json
    ├── checkpoint.ptnd
    └── journal/
~~~

A representação física pode ser simplificada inicialmente, mas a separação lógica permanece.

### metadata

Pode guardar:

- DocumentId;
- source location quando existe;
- base saved revision;
- latest recovery revision;
- timestamps;
- application/schema version;
- clean shutdown marker.

Não guardar secrets ou paths externos desnecessários.

## Checkpoint + journal

A estratégia recomendada é híbrida:

~~~text
periodic Recovery Checkpoint
+
small transaction journal after it
~~~

Checkpoint é snapshot consistente.

Journal registra alterações autorais posteriores de forma replayable.

Isso reduz tanto:

- custo de salvar snapshot completo a cada pequena edição;
- tamanho/tempo de replay de um journal infinito.

## Journal

**Journal** é uma sequência append-only de registros de recovery.

Append-only significa que novos registros são adicionados ao final sem reescrever entradas anteriores já confirmadas.

Direção:

~~~text
JournalHeader
TransactionRecord revision 101
TransactionRecord revision 102
TransactionRecord revision 103
CheckpointMarker ...
~~~

O formato de recovery pode reutilizar DTOs de Transaction quando estáveis, mas não precisa expor a representação interna exata de `HistoryEntry`.

## Durabilidade

Recovery não precisa fazer `fsync` em cada pointer move.

Somente commits autorais entram no journal.

A política de flush equilibra segurança e I/O:

~~~text
commit autoral
↓ append in-memory/buffered
↓ flush periódico ou por policy
~~~

Operações críticas como fechamento, troca de documento ou save podem solicitar flush imediato.

O intervalo exato é Application Setting e não fica congelado na arquitetura.

## Journal framing

Cada registro precisa ser detectável individualmente.

Direção:

~~~text
length
record type
revision
payload
checksum
~~~

Se crash interromper a última escrita:

~~~text
records anteriores válidos
+
último record truncado
~~~

loader descarta somente o record incompleto/corrompido final quando puder provar o boundary anterior.

Não interpretar bytes residuais como Transaction.

## Checksum

Recovery records usam checksum/hash para detectar corrupção acidental.

Isso não é mecanismo de autenticação.

Pode reutilizar BLAKE3/content hashing quando conveniente.

## Replay

Recovery:

~~~text
last valid checkpoint
↓
validate
↓
Transaction 101
↓
Transaction 102
↓
Transaction 103
↓
recovered Document
~~~

Cada Transaction passa novamente pelas invariantes compatíveis do Core.

Se um record não puder ser aplicado:

- interromper no último estado consistente;
- preservar os arquivos de recovery;
- produzir diagnóstico;
- nunca aplicar o restante assumindo que dependências continuam válidas.

## Recovery e migrations

Recovery data possui versionamento explícito.

Se a aplicação nova consegue migrar o checkpoint mas não o journal antigo, não deve inventar replay.

Preferir materializar checkpoint novo durante upgrades relevantes quando possível.

## Recovery e Resources

Mudanças em PixelLayer/resources grandes não devem copiar o blob inteiro em cada Transaction.

Recovery pode usar:

- COW tile blobs;
- content-addressed resource blobs;
- references por ContentHash;
- delta/replace de tile.

O journal referencia blobs já gravados de forma durável.

Record nunca aponta para temporário que pode desaparecer antes do replay.

## Recovery de documento nunca salvo

DocumentId existe desde a criação.

Assim recovery funciona mesmo sem filesystem path:

~~~text
Untitled Document
DocumentId D
↓ autosave
RecoveryRoot/D
~~~

Na restauração, a aplicação pode apresentar o documento como não salvo.

A UX dessa escolha será definida posteriormente.

## Clean shutdown

Ao fechar um documento com estado já salvo e sem recovery necessário:

~~~text
mark clean
↓
recovery pode ser removido
~~~

A remoção pode ser delayed/best-effort.

Na inicialização, somente recovery sem clean shutdown ou mais novo que o PTND confirmado precisa ser considerado candidato.

## Crash durante recovery write

O recovery store usa o mesmo princípio de não destruir a última base válida.

Novo checkpoint:

~~~text
checkpoint-old
↓
write checkpoint-new.tmp
↓ validate/flush
↓ replace checkpoint
↓ truncate/rotate journal somente depois
~~~

Nunca apagar o journal antigo antes de o checkpoint novo estar confirmado.

## Crash durante save principal

Save manual e recovery são independentes.

Se o PTND temp falha:

- arquivo principal anterior continua válido;
- recovery continua disponível;
- Document continua dirty;
- erro é reportado.

## External modification

Se o arquivo PTND no disco mudar externamente enquanto o documento está aberto, não sobrescrever silenciosamente.

A camada de I/O pode comparar:

- filesystem metadata;
- ContentHash do arquivo/manifest;
- saved file identity.

Se detectar mudança, retorna conflito de persistência.

A política de UX — reload, overwrite, save copy — fica para discussão posterior.

## File locking

A v0.1 não depende de lock exclusivo permanente como garantia de integridade.

Locks de plataforma podem ser advisory e inconsistentes em filesystems remotos.

Usar detecção de modificação externa + atomic replace como base.

Um lock cooperativo pode ser adicionado para melhorar experiência multi-instância, mas não substitui conflict detection.

## History não é Recovery

History existe para Undo/Redo na sessão.

Recovery existe para sobreviver a crash.

Não exigir que todo HistoryEntry permaneça para sempre apenas porque recovery precisa reconstruir estado.

O journal de recovery possui lifetime e pruning próprios.

## Privacy

Recovery pode conter o trabalho não salvo do usuário.

Portanto:

- fica em storage privado da aplicação;
- não é enviado por rede;
- não entra em telemetry;
- cleanup precisa existir;
- permissões do diretório devem ser restritivas quando a plataforma suportar.

## Limites

Recovery precisa de budget de disco.

Política pode considerar:

- número de documentos;
- idade;
- tamanho;
- clean/unclean;
- último acesso.

Nunca remover automaticamente o único recovery conhecido de documento dirty aberto.

Valores absolutos pertencem à aplicação/configuração.

## Testes

Casos obrigatórios:

- crash antes do primeiro checkpoint;
- crash durante append;
- record final truncado;
- checksum inválido;
- checkpoint novo falha antes de replace;
- save manual falha;
- document editado durante save;
- resource/tile recovery;
- schema migration;
- external modification;
- clean shutdown.

## Invariantes

1. PTND principal e Recovery Store são independentes.
2. Autosave não sobrescreve silenciosamente o save confirmado.
3. Save checkpoint registra exatamente a DocumentRevision gravada.
4. Edições ocorridas durante save permanecem dirty.
5. Recovery usa checkpoint + journal limitado.
6. Journal contém somente commits autorais, nunca pointer moves/previews.
7. Record incompleto final não invalida records anteriores.
8. Novo checkpoint só substitui o antigo depois de validado.
9. History e Recovery possuem lifetimes diferentes.
10. Resource/tile recovery evita cópias integrais desnecessárias.
11. Mudança externa de arquivo gera conflito explícito.
12. Recovery é dado privado local por padrão.

## Verificação do journal com replay (2026-10-10)

Escopo: framing com checksum, replay por prefixo confirmado e sessões de recovery. Revisão `a3e20794ebc79ea8774aa4b66de049e5d8ede080` sobre branch `petunia-design-rust`.

| Gate executado | Resultado |
|---|---|
| `cargo test --workspace` | pass: 349 passed / 0 failed (Engine com 7 de journal + 8 de recovery) |
| `cargo clippy --workspace --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |
| `node website/scripts/verify-progress.cjs` | pass |

Riscos/limites: blobs de resource viajam inline no record (COW/content-addressed é futuro); histórico de Undo não é restaurado pelo recovery; detecção de modificação externa e locks seguem pendentes.
