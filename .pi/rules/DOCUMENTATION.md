# Documentation Rules

Rules version: 1.0.0

These rules define the required structure, content, naming, and maintenance of project documentation.

## 1. Repository, scope, and language

- Host each project's documentation set in its own independent GitLab repository. The `docs/` directory named throughout these rules is the local checkout of that repository, not an ordinary source directory owned by the product repository.
- Keep the documentation repository's Git history, branches, access controls, CI, issues, and merge requests independent from the product repository.
- Record the documentation repository's GitLab URL and the associated product repository in `docs/README_ENG.md` and `docs/README_ITA.md`. Link the documentation GitLab project from the product repository's README as well.
- Keep one authoritative documentation repository per product. Reference it from other repositories instead of committing duplicated documentation copies.
- Target documentation commits and merge requests to the documentation GitLab repository, including when its local checkout appears at `<product>/docs/`.
- Write every reader-facing document in both English and Italian.
- Use `_ENG` for English and `_ITA` for Italian, immediately before the file extension: `SYSTEM_OVERVIEW_ENG.md` and `SYSTEM_OVERVIEW_ITA.md`.
- Treat the English version as the source of truth and keep the Italian version semantically aligned with it.
- Update both language versions in the same change whenever their content changes.
- Keep this policy and the other files under `rules/` in English only. They are internal agent instructions and are exempt from the bilingual naming requirement.

## 2. Directory layout

Every project must provide `docs/design/` and `docs/architecture/`. Add `docs/install/` for installation guidance and `docs/config/` when the product has configuration, logging, prompts, or `.nxmignore` behavior to document.

Use this layout as the baseline for the independent documentation repository:

```text
docs/
├── README_ENG.md
├── README_ITA.md
├── ROADMAP_ENG.md
├── ROADMAP_ITA.md
├── design/
│   ├── DATABASE_SCHEMA_ENG.md
│   ├── DATABASE_SCHEMA_ITA.md
│   ├── TECHNOLOGY_STACK_ENG.md
│   ├── TECHNOLOGY_STACK_ITA.md
│   ├── FUNCTIONAL_OVERVIEW_ENG.md
│   ├── FUNCTIONAL_OVERVIEW_ITA.md
│   └── assets/
├── architecture/
│   ├── SYSTEM_OVERVIEW_ENG.md
│   ├── SYSTEM_OVERVIEW_ITA.md
│   ├── CONTEXT_DIAGRAM_ENG.md
│   ├── CONTEXT_DIAGRAM_ITA.md
│   ├── COMPONENT_DIAGRAM_ENG.md
│   ├── COMPONENT_DIAGRAM_ITA.md
│   ├── DEPLOYMENT_ENG.md
│   ├── DEPLOYMENT_ITA.md
│   └── decisions/
├── install/
│   ├── HOW_TO_INSTALL_ENG.md
│   └── HOW_TO_INSTALL_ITA.md
└── config/
    ├── CONFIGURATION_ENG.md
    ├── CONFIGURATION_ITA.md
    └── config.example.<ext>
```

Create only documents applicable to the project. Do not create empty placeholders. In particular:

- Provide `DATABASE_SCHEMA_*` only when the project uses a database.
- Provide `HOW_TO_INSTALL_*` for every distributable application, service, or library. For a library, cover both consumer setup and development setup when they differ.
- Provide `CONFIGURATION_*` and a usable example whenever the product accepts configuration or supports `.nxmignore`.
- Add `GLOSSARY_ENG.md` and `GLOSSARY_ITA.md` when domain terminology needs stabilization.
- Keep living documents such as installation and configuration guides at stable paths without date prefixes.

## 3. Documentation index and traceability

`docs/README_ENG.md` and `docs/README_ITA.md` must:

- Explain the purpose and audience of the documentation set.
- Link to every maintained document in the corresponding language.
- Identify the documentation GitLab repository and the associated product repository, and explain how to clone the documentation locally.
- Link each English document to its Italian counterpart and vice versa.
- Identify generated artifacts and their sources when generation is used.

Keep documentation close to the behavior it describes. Any code, configuration, deployment, or architecture change must update the affected documents and diagrams in the same change.

## 4. Design documents

Design documents describe the current technical implementation.

### Database schema

When a database is used, document:

- The entity-relationship diagram.
- Entities, relationships, constraints, indexes, and relevant data types.
- Business invariants, deletion behavior, retention, and ownership rules.
- The migration files or schema definitions that remain authoritative for the physical schema.

Do not manually duplicate complete migration SQL when a reference to the authoritative migrations is sufficient.

### Technology stack

Document:

- Languages, runtimes, frameworks, databases, external services, and supported versions.
- Direct dependencies in a table containing name, purpose, and declared or resolved version.
- The manifest and lockfile that remain authoritative for exact and transitive dependency versions.

Refresh the dependency table whenever direct dependencies or supported platform versions change.

### Functional overview

Document the principal use cases and workflows, including actors, inputs, outputs, failures, and relevant state transitions.

For diagrams:

- Keep an editable source whenever the diagram tool provides one.
- Commit a readable SVG, PNG, or HTML export as appropriate.
- Provide `_ENG` and `_ITA` variants when the diagram contains language-dependent text; share one asset only when it is language-neutral.
- Use `diagram-design` for static architecture and technical diagrams and `archify` for interactive, explorable HTML when useful.
- Derive code-backed diagrams from repository evidence rather than intended but unimplemented behavior.

## 5. Architecture documents

Architecture documents explain system boundaries, component interactions, deployment, and the reasons behind significant choices.

Maintain, as applicable:

- `SYSTEM_OVERVIEW_*`: purpose, actors, system boundary, and primary components.
- `CONTEXT_DIAGRAM_*`: external actors and systems.
- `COMPONENT_DIAGRAM_*`: internal components, responsibilities, and interactions.
- `DEPLOYMENT_*`: environments, infrastructure, network boundaries, configuration sources, and release topology.
- `decisions/`: Architecture Decision Records containing context, decision, alternatives, consequences, and status (`proposed`, `accepted`, `deprecated`, or `superseded`).
- Security, scalability, reliability, performance, and observability requirements when they materially affect the architecture.

## 6. Installation guide

`docs/install/HOW_TO_INSTALL_ENG.md` and `docs/install/HOW_TO_INSTALL_ITA.md` must provide an executable procedure rather than a general description.

Include, as applicable:

1. Supported operating systems, architectures, runtimes, tools, and external dependencies, with versions.
2. Installation from the supported distribution channels and from source when supported.
3. Required directories, environment variables, credentials, migrations, and post-installation initialization.
4. Links to the configuration guide and example configuration.
5. Start, stop, and health-check commands.
6. A verification procedure with commands and expected results.
7. Upgrade, rollback, and uninstall procedures when relevant.
8. Troubleshooting steps that include the documented log directory and log file names.

Use copyable commands and test them against the supported environment. Record platform-specific differences instead of implying that one command works everywhere.

## 7. Configuration, coding-agent prompts, `.nxmignore`, and logs

`docs/config/CONFIGURATION_ENG.md` and `docs/config/CONFIGURATION_ITA.md` must cover all of the following topics.

### Configuration locations and precedence

- State the configuration file names, format, and exact locations for each supported operating system or deployment mode.
- Distinguish clearly among the installation directory, user configuration directory, system configuration directory, and product working directory.
- Document discovery order and precedence among defaults, configuration files, environment variables, and command-line options.
- Show how to select a non-default configuration path when supported.
- Identify ownership and permission requirements. Use placeholders for secrets and explain the supported secret-injection mechanism.

### Example configuration and option reference

- Maintain one complete, usable `config.example.<ext>` next to the configuration guides.
- Keep the executable example language-neutral; place detailed explanations in the bilingual guides.
- Match the example to the current schema, defaults, and accepted values. Validate it with available parsers, schemas, or tests.
- Explain every option's purpose, type, default, accepted values, constraints, and interactions with other options.
- Distinguish required options from optional ones and include realistic examples for common scenarios.

### Default coding-agent prompt

Provide at least one copyable default prompt that a user can give to a coding agent to create the configuration. The prompt must be adapted to the product and must instruct the agent to:

- Inspect the documented example, schema, and existing configuration before making changes.
- Create the configuration at the documented location with the documented file name and format.
- Use safe defaults and placeholders for secrets rather than inventing credentials.
- Create `.nxmignore` at the product working-directory root and populate it according to the documented syntax and purpose.
- Create or identify the documented log directory when required by the product, without assuming elevated permissions.
- Preserve existing user configuration and ask before overwriting files.
- Validate the resulting configuration and report every file and directory created or changed.

The English guide must contain the prompt in English and the Italian guide must contain its equivalent in Italian.

### `.nxmignore`

Document:

- Its purpose and which NXM tools or operations honor it.
- Why it belongs at the root of the product working directory and how that directory is determined.
- Its syntax, matching rules, precedence, and a practical example.
- Its relationship to `.gitignore` and any differences in behavior.
- Whether it is intended to be committed, generated, or locally customized.

Treat `.nxmignore` as an exclusion mechanism, not a security boundary. Base the documented behavior on the implementation rather than assumptions borrowed from `.gitignore`.

### Logs

Document:

- The exact log directory and log file names for every supported deployment mode.
- Whether paths are relative to the installation directory, product working directory, user directory, or system directory.
- The configuration option or environment variable that changes the log location.
- Log levels, formats, rotation, retention, and relevant permissions.
- Which log to inspect for startup, runtime, configuration, and integration failures.
- Sensitive-data handling and the safe way to share diagnostic logs.

Keep these paths consistent across the installation guide, configuration example, prompt example, and troubleshooting instructions.

## 8. Roadmap

Maintain `docs/ROADMAP_ENG.md` and `docs/ROADMAP_ITA.md` as living, macro-level documents. Use them to communicate direction and progress without duplicating the detailed issue tracker.

Include:

- The product goals and current development stage.
- Completed macro steps, with their completion date or release/version when available and links to relevant releases, ADRs, or documentation.
- Work currently in progress, at milestone rather than task granularity.
- Features already planned, with their purpose, expected user value, priority or time horizon, dependencies, and target release when one has actually been agreed.
- Features under consideration separately from committed work, so proposals are not presented as promises.
- Known constraints or major technical dependencies that affect the plan.

Use explicit statuses such as `completed`, `in progress`, `planned`, and `under consideration`. Keep completed milestones visible as project history. Update the roadmap whenever a macro step is completed, a feature is approved, or priorities and target releases change. Both language versions must describe the same status and plans.

## 9. Completion criteria

Documentation work is complete only when:

- Documentation changes belong to the independent documentation GitLab repository and no duplicate copy has been added to the product source repository.
- Every changed reader-facing document has matching `_ENG` and `_ITA` versions with equivalent content.
- All index links and cross-language links resolve.
- File names, directories, commands, defaults, and examples match the implementation.
- The configuration example is valid and the default coding-agent prompt produces files in the documented locations when followed.
- Installation commands and verification steps have been checked against the supported environment, or any unverified platform-specific step is explicitly identified.
- Diagram sources and exports reflect the current system.
- Direct dependency versions and supported runtime versions are current.
- The roadmap reflects completed macro steps and the currently approved feature plans.
- No unrelated documentation has been rewritten.
