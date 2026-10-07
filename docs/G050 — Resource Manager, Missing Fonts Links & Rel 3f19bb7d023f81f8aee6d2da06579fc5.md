# G050 — Resource Manager, Missing Fonts/Links & Relink/Embed Workflows

# Goal

Make external resources robust and repairable.

# Depends

G032, G034, G042, IO/platform services.

# Primary

editor-engineer + ui-component-engineer.

# Deliverables

Resource Manager panel/window; linked/embedded states; file grants; watcher; Relink/Embed/Unembed/Replace; Missing Fonts substitution UI; resource preflight rules.

# Acceptance

Moving/deleting image/font resources yields recoverable explicit state, not silent substitution; repair persists.

# Tests

same filename wrong file, changed hash, grants, font substitution, batch relink and offline external library.