# Commenting Guidelines

## Purpose
Comments should make complex mechanics easier to maintain by explaining **why** a rule exists and **how** it maps to Pokémon mechanics. Comments are not a change log.

## When to comment
- Explain non-obvious mechanics, game rules, or generational deltas.
- Clarify edge cases that are easy to regress.
- Explain precision-sensitive math or rounding decisions.
- Explain why a seemingly simpler implementation was rejected.

## When **not** to comment
- Do not narrate your editing process or deliberations.
- Do not include “history” notes (e.g., “changed from X”).
- Do not leave commented-out code; delete it or move it to a branch.
- Do not restate the code in plain English.

## TODO policy (keep as-is, but make them useful)
- Keep existing TODOs.
- New TODOs must be concise and action-oriented.
- If a TODO is large or cross-cutting, reference an issue or task tag.
- Avoid rambling TODOs; use short scope statements.

## Math & precision notes (required for precision-critical logic)
Any code that applies a fixed-point modifier or rounding must include a short explanation:
- State the fixed-point scale (4096 = 1.0x).
- Note how rounding is performed (use `apply_modifier()` and its rounding behavior).
- If an approximation is used, explain why it is acceptable.

## Mechanics & generations
- Gen 9 is the baseline. When implementing older generations, add comments describing the delta and why it differs.
- Keep generational mechanics notes near the logic they affect.

## Tests
- Comment the intent of a test (what rule it verifies), not the step-by-step code.
- Use short single-line comments above the assertion block.

## Formatting
- Keep comments short (1–3 lines) and specific.
- Prefer complete sentences and avoid colloquial phrasing.
- Use doc comments (`///`) for public APIs and fields.

## Review checklist (lightweight)
- Does the comment explain a non-obvious rule or constraint?
- Is it free of meta narration or change history?
- Is precision-critical math explained with 4096-scale and `apply_modifier()`?
- Would a new contributor understand the intent from this comment alone?

## Example transformations
- Replace “Let’s do X…” with “We must do X because Gen N rule Y.”
- Replace “Changed from Z” with a short rationale (“Gen 9 uses Z due to …”).
