# 19.23 — Data Merge Panel

# Identity

PanelId [ptnd.panel.data](http://ptnd.panel.data)_merge.

# Sections

Data Sources, Fields/Schema, Bindings, Preview Record and Issues.

# Source model

Provider/name/fingerprint/status/row count/schema. Refresh is explicit and reports changed fields.

# Fields

Searchable list with inferred/declared type, sample value and null/error indicators. Drag field onto bindable canvas/property can create binding.

# Bindings

Rows target object/property and expression/fallback. Invalid/missing field clearly flagged.

# Preview

Record navigator index/search/previous/next. Preview override state is derived; panel shows “Previewing record N” banner.

# Generate

Opens generation/preflight flow with target mode, output count, filenames and estimated resources.

# Accessibility

All drag operations have Add Binding dialog/action alternatives.

# Tests

100k rows, schema refresh, field removal, preview no mutation, generation cancel and plugin data provider.