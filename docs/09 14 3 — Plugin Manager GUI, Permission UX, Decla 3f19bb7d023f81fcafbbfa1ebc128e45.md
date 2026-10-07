# 09.14.3 — Plugin Manager GUI, Permission UX, Declarative Panels & Developer Tooling

# Plugin Manager layout

Left/filter list installed plugins; main detail shows icon/name/version/author/source/status; tabs Overview, Permissions, Contributions, Logs/Diagnostics, Updates.

# Install

Select package/approved source -> verify -> summarize compatibility and requested permissions -> user grants/denies optional scopes -> install disabled/enabled policy.

# Permission groups

Document, Files, Network, Clipboard, UI/Tools, Background, Administration. Each row states reason supplied by plugin and current grant. High-risk changes highlighted.

# Revocation

User can revoke capability; broker enforces immediately or after documented restart if technically necessary. UI shows functionality likely affected.

# Crash state

Plugin row indicates Crashed/Disabled/Incompatible/Permission blocked. Actions: Restart, View Diagnostics, Disable, Remove. Main app remains responsive.

# Contributions

List Actions, Panels, Tools, Formats, Data Sources with stable IDs. Useful for auditing conflicts and command discovery.

# Declarative panel UX

Plugin panel receives normal docking, theme, density, accessibility and keyboard behavior automatically. Plugin controls cannot choose arbitrary fonts/colors outside safe style tokens except content preview areas explicitly allowed.

# Developer mode

Load unpacked plugin, hot-restart process, schema/type validation, mock grants, event/RPC inspector, sample document, test runner and packaging/signing command.

# Marketplace/source

If future marketplace exists, provenance/rating are separate from permission trust. Sideloaded package gets clear source warning, not blanket prohibition.

# Accessibility

Manager and plugin panels expose same Qt semantics; manifest provides TextIds/labels/descriptions and host validates missing accessible names.