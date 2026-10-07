# 10.11.1 — Data Merge Expression Language Grammar, Types & Sandbox

# Goal

Useful deterministic expressions without Python/JavaScript execution.

# Grammar baseline

Literals string/number/bool/null/date optional. Field reference field("name") or concise ${field} compiled to AST. Operators + - * / %, comparisons, and/or/not, ternary/if. Allowlisted functions: concat, upper/lower/title, trim, format_number, format_date, coalesce, substring, replace fixed-string, round, min/max and explicit color constructors if supported.

# No capabilities

No loops, recursion, arbitrary object traversal, filesystem, network, process, environment, eval/reflection or user-defined code.

# Types

String/Number/Bool/Date/Color/ResourceRef/Null. Binding target validates output compatibility.

# Limits

AST nodes/depth, output length, function cost and evaluation time bounded. Regex excluded unless safe bounded engine is chosen.

# Errors

Parse/type/evaluation error has position, code, BindingId/record and recovery context.

# Versioning/tests

ExpressionVersion stored. Test Unicode, divide-by-zero, nulls, huge strings, malicious payload and locale formatting.