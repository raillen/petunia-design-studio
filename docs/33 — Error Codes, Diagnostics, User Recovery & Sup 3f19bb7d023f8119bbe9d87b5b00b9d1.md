# 33 — Error Codes, Diagnostics, User Recovery & Support Taxonomy

# Purpose

Replace opaque exceptions/strings with stable structured diagnostics.

# Domains

Core validation, geometry, raster, text/font, color/profile, render/GPU, PTND/IO, import/export, resource/files, plugin, MCP, jobs, UI/platform, security and release/update.

# Error contract

ErrorCode, category/severity, technical fields, safe user TextId/message parameters, recoverable/retryable, affected IDs/path, suggested ActionId/help, correlation ID and privacy classification.

# Rule

Internal exception text can be logged but is not API/UI contract. Plugin/MCP receive stable codes; UI maps to concise recovery-oriented message.