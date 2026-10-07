# G05 — Actions, Commands, Transactions & History Kernel

# Goal

Make all document mutation semantic, transactional and undoable.

# Depends

G04.

# Authority

09.3, 21.1–21.5.

# Owner

editor-engineer + architect. Reviewer quality-reviewer.

# Deliverables

ActionId metadata skeleton; Command variant/interface; TransactionBuilder; DocumentMutator; ChangeSet; HistoryManager; undo/redo; coalescing hooks; revision numbers.

# Acceptance

Transform/property/create/delete/reparent sample commands atomic; rollback on failure; undo/redo returns semantic snapshot exactly; nested transaction rules enforced; new edit after undo clears redo.

# Evidence

Property tests, transaction fault injection and memory accounting baseline.