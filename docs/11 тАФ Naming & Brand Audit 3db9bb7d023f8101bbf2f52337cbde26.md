# 11 — Naming & Brand Audit

<aside>
🌸

**Status:** **Aubrieta Design is now the selected canonical product name.** This audit remains a preliminary public-web collision review, not legal trademark clearance. Final legal/trademark clearance is still required before public commercial release.

</aside>

## Finalists audited

| Candidate | Brand character | Collision/risk found | Current assessment |
| --- | --- | --- | --- |
| **Aubrieta Design** | delicate, artistic, premium, botanical | No major creative-software product surfaced in the current search. There is an active AUBRIETA LIMITED in the UK in market research, an older US AÚBRIETA trademark application in an unrelated fumigation class, and an Aubrieta typeface. Botanical use is naturally widespread. | **Strongest finalist — low-to-moderate preliminary naming risk for creative software.** |
| **Rayen Design** | short, soft, contemporary, South-American botanical meaning | Highly crowded: Rayen Soft, Rayen Digital, Rayen Salud, [Rayen.com](http://Rayen.com), the Python project/package RAYEN, several apps, companies and registered RAYEN marks in unrelated classes. | **Remove from primary shortlist.** Too much technology/software collision and search noise. |
| **Potyra Design** | Brazilian, floral, organic, distinctive | A business already appears literally as **Paulina Potyra Design**, with `potyra.com`; Potyra is also used by Brazilian companies/products and appears in historical INPI records. | **Remove from primary shortlist for this exact “Potyra Design” form.** Strong cultural fit but direct naming collision. |
| **Blomia Design** | soft, invented-sounding, modern | `blomia.com` is used by an active Spanish plant-propagation company; a previous Brazilian Blomia fintech also existed. More importantly, **Blomia** is already the biological genus of the tropical house-dust mite *Blomia tropicalis*, strongly associated with allergy literature. | **Remove from primary shortlist.** The mite/allergen association is a poor long-term semantic anchor. |

## File-extension scan

No prominent established file-format collision surfaced for `.aubrieta` or `.aubri` in the public extension-search pass. `.aubrieta` is therefore the **canonical** native suffix and `.aubri` an **accepted short alias** for the same package/schema. `.abrt` is rejected despite no major file-format collision because **ABRT** is already the established Automatic Bug Reporting Tool name in the Fedora/RHEL ecosystem, which would create Linux/search/documentation ambiguity. This remains a practical collision scan, not a global extension reservation system.

## Domain observations

- `rayen.com` is occupied by a long-established Spanish company.
- `blomia.com` is occupied by Blomia S.A. in Spain.
- `potyra.com` is associated with Paulina Potyra Design and is reported registered.
- No authoritative availability conclusion for `aubrieta.com`/`aubrieta.design` was established from indexed web sources in this pass; registrar/RDAP confirmation is required before purchase.

## Pronunciation and international usability

- **Aubrieta** — elegant and memorable, but pronunciation varies somewhat across languages. It remains readable in Portuguese, English, Spanish and French and visually looks like a premium creative brand.
- **Rayen** — easiest pronunciation, but overused.
- **Potyra** — easy in Portuguese/Spanish, visually distinctive, but the `y` may invite variant spellings (`Potira`) and direct “Potyra Design” collision is material.
- **Blomia** — very easy to pronounce, but the biological/allergy association outweighs the phonetic advantage.

## Final naming decision

1. **Aubrieta Design — selected canonical product name.**
2. Potyra Design — rejected for direct/near-direct design naming collision.
3. Rayen Design — rejected as too crowded in technology/software.
4. Blomia Design — rejected due to commercial collisions plus mite/allergen association.

## Required pre-release legal/brand gate

The product is now named **Aubrieta Design**, but before public commercial release perform deeper clearance:

- official trademark databases in Brazil, US, EU and WIPO/Madrid;
- Nice classes most relevant to downloadable software, SaaS, design/creative services and digital goods;
- registrar/RDAP checks for priority domains;
- GitHub organization/repository naming, [crates.io](http://crates.io), package registries and major app stores;
- social-handle availability;
- pronunciation/negative-meaning review in PT-BR, EN, ES and FR;
- final pre-release recheck of `.aubrieta` and `.aubri` associations plus MIME/OS registration behavior.

The project/notebook may use Aubrieta Design immediately. Native format policy is also resolved at architecture level: **`.aubrieta` canonical + `.aubri` accepted short alias**. Legal/domain clearance remains a release gate rather than a blocker for internal development.

# Canonical brand/technical naming contract

This page is a **brand/release gate**, not a source of engine behavior. Code agents use these names consistently:

- **Official product name:** `Aubrieta Design`.
- **Permitted short product reference in prose/UI where context is unambiguous:** `Aubrieta`.
- **Canonical native extension:** `.aubrieta`.
- **Accepted short native alias:** `.aubri`.
- **Canonical document format ID:** `org.aubrieta.design.document`.
- **Built-in semantic namespace prefix:** `aubrieta.*`.
- **Rust crate prefix:** `aubrieta_` for libraries and `aubrieta-` for application binaries/packages where Cargo/package conventions justify it.
- **CLI executable/name:** `aubrieta` unless a later packaging ADR requires a platform-specific launcher alias.

Do not reintroduce `Petunia Design`, `VectorVonDoom`, `vd_*` or `petunia.*` into new public/canonical implementation surfaces. Historical/migration references must be labeled as such.

# Brand resource separation

Brand identity assets (application logo, wordmark, marketing illustrations, website assets) are separate from semantic UI IconIds. Product-brand colors may seed default theme primitives, but feature code must never hard-code brand colors. Themes/resource packs remain free to alter UI presentation while preserving Aubrieta identity/legal assets where required by distribution policy.

# Legal/trademark wording

Internal documentation may use Aubrieta Design immediately. Public releases must not claim trademark registration/status that has not been legally established. The legal clearance gate is a release-management obligation and does not authorize code agents to rename product/package/file identifiers opportunistically.

# Rename discipline

Any future product rename is a high-impact ADR/migration project covering package IDs, MIME/associations, native format aliases, CLI, crates, semantic namespaces, VitePress, installer metadata, update channels and external API compatibility. A marketing rename must not be applied as a repository-wide search/replace without a compatibility plan.