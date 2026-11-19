# Commit, Version and Changelog Rules

These rules are mandatory for every commit and release in this repository.

## 1. Conventional Commits

Every commit message MUST follow the Conventional Commits format:

```
<type>(<scope>): <description>
```

Examples: `fix(server): handle malformed JSON-RPC id`, `docs: update README`.

- `scope` SHOULD name the affected crate (`server`, `config`, `logs`) when the change touches only one crate; omit it when the change touches several.
- Mark breaking changes with `!` after the type/scope, or with a `BREAKING CHANGE:` footer: `feat(config)!: drop TOML-only config`.

### Version impact per type

| Type | Bump | Example |
| --- | --- | --- |
| `feat` | minor — `0.1.1` becomes `0.2.0` | new functionality |
| `fix` | patch — `0.1.0` becomes `0.1.1` | bug fix |
| breaking (`!` / `BREAKING CHANGE:`) | next major — in 0.x this is the minor: `0.1.0` becomes `0.2.0` | incompatible change |
| `docs`, `chore`, `test`, `refactor`, `ci` | none | no release |

## 2. Versioning

- The crate version lives in its `Cargo.toml` (`version = "..."`).
- Every `feat` or `fix` commit bumps ONLY the version of the crate it touches; the other crates stay where they are.
- Bump the version in the SAME commit as the change, so history and `Cargo.toml` never disagree.
- Sequential within the level: fixes go `0.1.0 → 0.1.1 → 0.1.2`; a feature resets the patch: `0.1.1 → 0.2.0`.
- Commit types with no bump never change any version.

## 3. CHANGELOG.md

- Every crate MUST keep a `CHANGELOG.md` next to its `Cargo.toml`.
- One section per released version, newest first: `## <version> - <YYYY-MM-DD>`.
- Entries are the Conventional Commit messages of the changes in that version, one `-` bullet each.
- Only types that bump the version (`feat`, `fix`, breaking) are recorded; `docs`, `chore`, `test`, `refactor` and `ci` are not.
- Update the CHANGELOG in the SAME commit as the version bump.

## 4. Pre-commit checklist

Before committing:

- [ ] Commit message matches the Conventional Commits format.
- [ ] The affected crate's version was bumped (for `feat`, `fix` or breaking changes).
- [ ] `CHANGELOG.md` was updated when the version was bumped.

## 5. Documentation

- Public APIs: `///` doc comments with at least one example.
- End of session: update `CONTEXT.md` and `CHANGELOG.md`.
- Every design/analysis/plan doc lives in `nxm-docs/` (or inside the product if it concerns a single product), bilingual (`_ITA` / `_ENG`).
- The code repo keeps only RULES/CONTEXT/STATUS/CHANGELOG.
