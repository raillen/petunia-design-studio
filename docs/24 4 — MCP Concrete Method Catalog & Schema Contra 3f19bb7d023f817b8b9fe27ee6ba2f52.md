# 24.4 — MCP Concrete Method Catalog & Schema Contract

# App/session

app.get_info

app.list_capabilities

sessions.list

sessions.get_active

sessions.activate_view

# Document

document.summary

document.get_metadata

document.validate

document.preflight

[document.save](http://document.save)

[document.save](http://document.save)_as

document.create

document.close

# Objects

objects.query

objects.get

objects.get_bounds

objects.describe_type

# Selection

selection.get

selection.set

selection.clear

# Actions/properties

actions.list

actions.describe

actions.execute

properties.describe

properties.get

properties.set

# Transactions

transactions.dry_run

transactions.execute

# Import/export

formats.list

import.analyze

import.start

export.analyze

export.start

export.quick

# Jobs

jobs.get

jobs.list

jobs.cancel

jobs.subscribe/events according transport.

# Plugins/resources

plugins.list_contributions

plugins.get_status

resources.list

resources.describe

resources.request_file_grant

# UI

ui.get_state

ui.inspect

ui.list_panels

ui.focus

Developer scopes: ui.screenshot, ui.synthetic_input.

# Diagnostics

diagnostics.get_summary

diagnostics.get_event_codes/help.

# Schema rule

Every method has JSON Schema, scope, side effect classification, revision behavior, pagination/job behavior, deterministic errors and examples.