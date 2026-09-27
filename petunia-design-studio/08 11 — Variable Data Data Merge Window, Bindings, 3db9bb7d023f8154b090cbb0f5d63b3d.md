# 08.11 — Variable Data / Data Merge Window, Bindings, Preview & Generation

# Role

Data Merge is a first-class Design subsystem for variable-data design. It is powerful enough for cards, labels, badges, certificates, catalog tiles and personalized graphics without creating a full Publisher Persona.

# Entry points

- Design panel `Data` / `Data Merge`;
- menu `Data > Data Merge…` or equivalent semantic location;
- command palette;
- context actions on text/image objects: `Bind to Data Field…`.

# Data Merge panel

Persistent dockable panel for everyday binding/preview.

Sections:

1. Source;
2. Fields;
3. Current Record;
4. Bindings summary;
5. Preview/Generate actions.

# Source section

Shows source type, file/service name, status and record count.

Actions:

- Attach CSV/TSV/JSON;
- Reload;
- Replace Source;
- Data Source Settings;
- Detach.

SQLite/API/online-sheet sources are **Post-V1 Candidate** adapters. They must use the same `DataSourceAdapter`/panel semantics and cannot introduce provider-specific binding models.

# Import data source dialog

Preview table of first records with parsing options:

- delimiter;
- header row;
- encoding detection/override;
- date/number locale;
- empty value policy;
- image path base directory;
- JSON root path/mapping where relevant.

Invalid rows/cells display warnings with counts and downloadable/reviewable details.

# Fields list

Each field row shows type icon + field name. Types may include Text, Number, Date, Boolean, Image/File, URL. Dragging a field onto canvas/object may create or bind content where unambiguous.

# Binding workflow

Selection + field → `Bind`. Binding target property selector appears when object has multiple meaningful properties.

Examples:

- TextObject.content ← `FirstName`;
- ImageObject.source ← `PhotoPath`;
- **Post-V1 Candidate unless promoted by 10.11:** Object.visibility ← condition/expression such as `Discount > 0`;
- **Post-V1 Candidate unless promoted:** Fill/Color binding when an explicit typed converter exists.

V1 binding examples should focus first on Text content and Image source replacement, matching the Persona scope.

# Binding badge

Bound objects display small non-printing canvas badge/overlay when Data Bindings overlay enabled. Layers row can show Data badge. Hover explains field mapping.

# Binding inspector

For selected bound object:

- source field;
- target property;
- formatter;
- fallback/default;
- null/empty policy;
- transform/expression;
- detach binding.

# Formatters

Text: prefix/suffix, case transform, truncate/ellipsis rules.

Number: decimal places, grouping, percentage, currency.

Date: locale/date format.

Image: fit/fill behavior, missing-image fallback.

# Expression/condition UI

Start visually simple:

- field;
- comparator;
- value/field;
- action such as Visibility.

Advanced textual expressions are a **Post-V1 Candidate**. V1 may use simple schema-driven field/formatter bindings; if textual expressions are introduced later they require a side-effect-free grammar, validation, autocomplete, deterministic evaluation and a separate security/complexity review.

# Record preview

Panel provides Previous/Next arrows, record number field and total count. Changing preview updates evaluated document without permanently materializing record content.

Preview must clearly indicate **Data Preview Mode** so user does not confuse record values with literal document content.

# Dedicated Data Merge window

For large workflows, open resizable subsystem window:

- left source/field mapping;
- center data table;
- right binding/errors/preview summary;
- top filters/search;
- bottom Generate/Export actions.

# Data table

Virtualized; sortable/filterable without changing source unless explicitly saved. Row status can show validation errors, missing images and excluded records. Multi-select records for subset generation.

# Generate dialog

Choices:

- Create Surfaces in current document;
- Create new Aubrieta document;
- Export directly;
- one file per record;
- one multipage PDF/document where format supports;
- selected/all/filtered records.

# Generation naming

Uses filename/template token editor. Collision policy defined before generation.

# Overflow handling

Text overflow policy is explicit:

- warn only;
- auto-fit within allowed limits;
- clip;
- expand frame where layout permits.

Preflight summary counts records with overflow/missing assets before final generation.

# Data errors

Never silently skip malformed records. Error panel groups by row/field/type and allows filter to affected records. Direct export can continue valid records only if user chooses policy explicitly.