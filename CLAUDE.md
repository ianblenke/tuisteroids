<!-- OPENSPEC:START -->
# OpenSpec Instructions

These instructions are for AI assistants working in this project.

Always open `@/openspec/AGENTS.md` when the request:
- Mentions planning or proposals (words like proposal, spec, change, plan)
- Introduces new capabilities, breaking changes, architecture shifts, or big performance/security work
- Sounds ambiguous and you need the authoritative spec before coding

Use `@/openspec/AGENTS.md` to learn:
- How to create and apply change proposals
- Spec format and conventions
- Project structure and guidelines

Keep this managed block so 'openspec update' can refresh the instructions.

<!-- OPENSPEC:END -->

# Development Methodology: Spec-First TDD

This project enforces a strict spec-first, test-driven development workflow. **These rules are non-negotiable.**

## The Iron Rules

1. **No code without a spec.** Every feature, behavior, and requirement MUST be specified in an OpenSpec spec before any implementation begins.
2. **No code without a failing test.** Tests MUST be generated from spec scenarios. Every `#### Scenario:` produces at least one test. Run tests first to confirm they fail.
3. **100% spec coverage.** Every requirement and scenario in specs MUST have corresponding test(s). No scenario left untested.
4. **100% code coverage.** Every line and branch of implementation code MUST be exercised by tests. No untested code paths.
5. **TDD cycle is mandatory.** Red (failing test) → Green (make it pass) → Refactor. No exceptions.

## Workflow Order

```
Spec → Tests (from spec) → Verify tests fail → Code (to pass tests) → Verify coverage → Done
```

## What to Refuse

- Writing implementation code without a backing spec
- Writing tests without a spec scenario driving them
- Merging code with less than 100% coverage
- Skipping the "verify tests fail" step