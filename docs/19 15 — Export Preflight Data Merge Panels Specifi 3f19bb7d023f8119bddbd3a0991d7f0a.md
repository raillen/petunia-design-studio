# 19.15 — Export / Preflight / Data Merge Panels Specification

# Export

Target/slice list, preset selector, output variants/scales, folder/grant status, quick export and open full Export dialog.

# Preflight

Issue list grouped severity/type/Surface, filter/search, navigate to object, deterministic Fix action, rerun/status.

# Data Merge

Source selector/schema fields, bindings list, record navigator, refresh, preview toggle and Generate action.

# Jobs

Long generation/export represented through shared Job center; panels show relevant job progress without duplicating scheduler.

# Model/view

All potentially large lists use QAbstractItemModel; issues/rows paginated/virtualized.

# Accessibility/tests

Issue navigation keyboard, binding errors, export target selection, large 100k-row Data Merge and cancellation.