# AGENTS.md

## Tooling and MCP Usage

When working in this repository, use the following tools whenever they are available.

### Codebase Memory MCP

Use **Codebase Memory MCP** to retrieve and preserve useful knowledge about the repository across tasks.

Use it for:

- Understanding existing architecture, conventions, and important implementation decisions.
- Looking up previously discovered information about the codebase before re-investigating it manually.
- Finding relationships between modules, services, components, and important files.
- Recalling previous fixes, refactors, design decisions, and known constraints.
- Storing durable, repository-specific knowledge that will likely be useful in future tasks.

Before performing broad codebase exploration, check Codebase Memory first when available.

Do not blindly trust stale memory. If remembered information conflicts with the current code, the current code is the source of truth. Update or correct the stored knowledge when appropriate.

### Serena

Use **Serena** for semantic exploration and modification of the codebase.

Prefer Serena over raw text search or reading entire files when the task involves code structure.

Use it for:

- Finding symbols, classes, functions, methods, interfaces, and their definitions.
- Finding references and usages of symbols.
- Understanding relationships between symbols and modules.
- Navigating unfamiliar parts of the repository.
- Inspecting only the relevant portions of large files.
- Performing precise, symbol-level code edits and refactors when supported.
- Identifying the correct implementation location before making changes.

Avoid reading large files from top to bottom if Serena can retrieve the relevant symbol or section directly.

For repository exploration, prefer this order when practical:

1. Check Codebase Memory for existing repository knowledge.
2. Use Serena to locate and inspect relevant symbols and references.
3. Use regular file search / grep / direct file reads only when needed.

### Context7

Use **Context7** whenever external library, framework, SDK, API, or tool documentation is relevant and Context7 is available.

Use it for:

- Retrieving up-to-date documentation for dependencies used by the repository.
- Verifying current APIs, configuration options, supported behavior, and recommended patterns.
- Checking version-specific documentation before implementing code that depends on third-party packages.
- Looking up examples for unfamiliar libraries or APIs.
- Avoiding assumptions based on outdated model knowledge.

Prefer Context7 documentation over relying solely on memory when working with third-party APIs.

When possible, identify the dependency and its version from the repository first, then consult documentation relevant to that version.

## General Tool Selection

Use the most specialized available tool for the task:

- **Codebase Memory MCP** → repository knowledge and persistent codebase context.
- **Serena** → semantic code navigation, symbol analysis, references, and precise code changes.
- **Context7** → external library/framework/API documentation.
- **grep / search / direct file reads** → fallback for textual searches, configuration files, generated files, or cases where semantic tools are insufficient.

These tools complement each other and should be combined when useful.

For example, when implementing a feature that depends on an existing subsystem and an external library:

1. Check Codebase Memory for known architecture and previous decisions.
2. Use Serena to inspect the existing subsystem and locate integration points.
3. Use Context7 to verify the external library's current API.
4. Implement the smallest appropriate change.
5. Run relevant tests, linters, and type checks.
6. Store important newly discovered architectural knowledge in Codebase Memory when it is likely to help future work.

## Availability and Fallbacks

Do not assume that every MCP server or tool is available in every environment.

If **Codebase Memory MCP**, **Serena**, or **Context7** is unavailable:

- Continue using the best available alternatives.
- Do not block the task solely because an optional tool is missing.
- Do not fabricate results from unavailable tools.
- Do not claim that documentation, memory, references, or symbols were checked unless they actually were.

If tool availability can be inspected, check it before assuming a tool is missing.

## Source of Truth

Use the following priority when information conflicts:

1. Current repository code and configuration.
2. Repository tests and executable behavior.
3. Version-specific official documentation retrieved through Context7 or another authoritative source.
4. Codebase Memory.
5. General model knowledge.

Repository memory and external documentation are aids; they do not override the actual state of the checked-out codebase.

## Working Principles

Before modifying code:

- Understand the relevant implementation and surrounding architecture.
- Search for existing patterns before introducing new abstractions.
- Check usages and references before changing public symbols or behavior.
- Verify third-party APIs instead of guessing them.
- Prefer focused changes over unrelated refactors.

After modifying code:

- Run the most relevant tests and validation commands available.
- Check affected references and call sites when appropriate.
- Keep existing repository conventions unless there is a clear reason to change them.
- Record durable architectural discoveries in Codebase Memory when available.
