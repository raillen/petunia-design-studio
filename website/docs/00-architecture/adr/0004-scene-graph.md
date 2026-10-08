# ADR-0004 — SceneGraph autoral ordenado

**Status:** Accepted

## Contexto

A cena precisa preservar hierarquia, z-order, transforms, bindings, symbols e ownership sem virar draw list nem ECS genérico.

## Decisão

A v0.1 usa:

~~~text
ObjectId → SceneNode lookup
+
Vec<ObjectId> ordenado em Page/containers
~~~

`ParentRef` torna ownership estrutural explícito. Lookup e ordem autoral são conceitos separados.

## Alternativas rejeitadas

**Ordem derivada de HashMap:** não determinística e semanticamente incorreta.

**ECS como requisito inicial:** complexidade sem evidência de necessidade.

**Generational Arena obrigatória:** permanece otimização possível após profiling; runtime handle nunca substitui ObjectId.

**Render graph como SceneGraph:** mistura fonte autoral e estado derivado.

## Consequências

- parent/children mudam atomicamente;
- z-order vem dos vectors ordenados;
- world transform é derivado;
- storage pode evoluir sem mudar PTND;
- cycles são rejeitados no Core;
- Symbol/Clip/Mask usam relações explícitas.

## Referências

[scene.rs](#/docs/01-core/scene.md)
