# AGENTS.md

Guidance for AI agents working in this repository.

## What this project is

`newt` is a terminal UI wizard (Rust, ~1,800 lines) that scaffolds `.devcontainer` environments. The user runs `newt <project-name>`, answers 5 checklist screens, and gets a complete `.devcontainer/` tree plus `.gitignore` written to disk. No network calls, no external APIs — all generation is local string formatting.

## Build and test

```bash
cargo build --release          # produces target/release/newt
cargo test                     # unit tests across all modules
./install.sh                   # installs binary to ~/.local/bin (or first arg)
```

No code generation steps, no migration scripts, no external services needed.

## Architecture

Data flows in one direction:

```
main.rs          → validates CLI args, resolves output path
app/wizard.rs    → terminal event loop, returns Option<Answers>
app/state.rs     → App struct holding SelectLists per wizard step
app/types.rs     → SelectItem, SelectList, Step enum, Answers struct
generator/mod.rs → generate(answers, output_dir) writes files to disk
generator/config.rs  → Answers → Config (typed sets: LanguageSet, ToolSet, …)
generator/templates.rs → Config → rendered file strings
ui.rs            → ratatui rendering (called from wizard event loop)
```

`Answers` is the seam between the wizard and the generator. It holds raw `Vec<&'static str>` labels from each checklist. `Config` converts those labels into typed boolean/set fields that the template functions consume.

## Wizard options (source of truth: `src/app/state.rs`)

| Step | Options | Pre-selected |
|------|---------|--------------|
| Languages | Swift, Rust, Python, Node.js/Bun, Go, Ruby, Java/JVM | — |
| Databases | PostgreSQL, MySQL/MariaDB, SQLite, Redis, MongoDB | — |
| AI Tools | Claude Code, pi, GitHub Copilot, OpenCode | Claude Code |
| Extra CLI | ripgrep, fd-find, jq, bat, htop, httpie, just, watchexec | ripgrep, fd-find, jq, bat |
| Gitignore | .env, .env.local, *.log, .DS_Store, *.tmp/*.swp, dist//build/, target/, node_modules/, .venv//venv/, __pycache__//*.pyc, .idea//.vscode/ | first six |

## How to add a new language or tool

Adding a language (the same pattern applies to databases, AI tools, and CLI tools):

1. **`src/app/state.rs`** — add a `SelectItem::new(...)` (with `.on()` if it should default to selected) to the relevant `SelectList` in `App::new()`.
2. **`src/generator/config.rs`** — add a field to the relevant `*Set` struct and populate it in `*Set::from_labels()`.
3. **`src/generator/templates.rs`** — add the install commands / config lines wherever that tool is conditionally rendered.
4. **Firewall domains** — if the tool needs network access, add its domains to the relevant domain list in `config.rs` so `init-firewall.sh` allows them.
5. Run `cargo test` — the template tests in `generator/mod.rs` will catch missing cases.

## Key invariants

- **Labels are `&'static str` identity keys.** `SelectItem::label` is used as the lookup key in `from_labels()`. If you rename a label in `state.rs`, update the matching string in `config.rs`.
- **`generate()` is pure after `Answers` is collected.** It does file I/O but no terminal interaction. Safe to call in tests with a temp dir.
- **Terminal state is always restored.** `wizard.rs` wraps the event loop in a cleanup block that calls `terminal::restore()` even on panic. Don't add `process::exit()` calls that would bypass this.
- **Minimum terminal size is 60×15.** `ui.rs` renders an error widget below this threshold instead of the normal UI.

## Testing approach

Tests live alongside their modules (not in a separate `tests/` tree):

- `src/app/wizard.rs` — step navigation logic
- `src/generator/config.rs` — label → config mapping
- `src/generator/mod.rs` — full `generate()` round-trips writing to a temp dir

Tests are integration-style: they call public functions with realistic inputs and assert on outputs. No mocking of the file system — `generate()` tests write to `tempdir()`.

## Generated file inventory

| File | Template function |
|------|------------------|
| `.devcontainer/devcontainer.json` | `templates::devcontainer_json()` |
| `.devcontainer/Dockerfile` | `templates::dockerfile()` |
| `.devcontainer/install-ai-tools.sh` | `templates::install_ai_tools_sh()` |
| `.devcontainer/init-firewall.sh` | `templates::init_firewall_sh()` |
| `.devcontainer/bashrc.sh` | `templates::bashrc_sh()` |
| `.gitignore` | `templates::gitignore()` |

## CLI interface (`src/main.rs`)

```
newt <project-name> [-o <output-dir>]
```

- Project name is validated: no spaces, no dots, no special characters.
- If `.devcontainer/` already exists in the output path, the user is prompted to confirm overwrite before the wizard opens.
- On wizard cancellation (Esc or `q`), no files are written and the directory is not created.

## Dependencies

Three direct dependencies, all stable and low-churn:

| Crate | Version | Purpose |
|-------|---------|---------|
| `ratatui` | 0.29 | Terminal UI widgets and layout |
| `crossterm` | 0.28 | Cross-platform raw terminal control |
| `clap` | 4 | CLI argument parsing (derive feature) |

No async runtime, no proc-macro crates beyond clap-derive. Build times are fast.
