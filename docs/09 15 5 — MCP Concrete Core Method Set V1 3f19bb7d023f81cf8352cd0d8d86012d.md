# 09.15.5 — MCP Concrete Core Method Set V1

# Application

app.get_info

app.list_sessions

app.get_capabilities

# Document

document.get_summary

document.get_metadata

document.validate

document.preflight

[document.save](http://document.save)

[document.save](http://document.save)_as using FileGrant

document.create/new helper when scope permits

# Objects

objects.query

objects.get

objects.get_children

objects.get_bounds

objects.describe_type

# Selection

selection.get

selection.set

selection.clear

selection.query_candidates

# Actions

actions.list

actions.describe

actions.execute

# Properties

properties.describe

properties.get

properties.set

properties.set_many

# Transactions

transactions.dry_run

transactions.execute

# Resources

resources.list

resources.get_metadata

resources.relink through grant

resources.embed

# Import/export

import.list_formats

import.analyze

import.start

export.list_formats

export.analyze

export.start

# Jobs

jobs.get

jobs.list

jobs.cancel

# UI semantic

ui.get_state

ui.list_panels

ui.inspect

ui.focus_action/control in test/dev scope

ui.capture screenshot only developer scope

ui.synthetic_input only test/dev scope

# Plugin

plugins.list_contributions

plugins.get_status; admin enable/disable/update are separate high-risk scope and may remain outside V1 MCP.

# Helpers

text.create_frame; vector.create_shape; photo.add_adjustment etc are ergonomic wrappers generated/delegated to Action schemas, not duplicate mutation engine.

# Rule

Every concrete method gets machine-readable JSON Schema generated in repository and cookbook test before V1 API freeze.