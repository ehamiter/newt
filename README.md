# newt (`new` projec`t`)

A terminal UI wizard for scaffolding new projects with a working `.devcontainer` setup.

```
~/projects: $ newt foobar
```

This creates `~/projects/foobar/` and walks you through a series of questions to produce a lean, fully configured devcontainer tailored to your project.

## What it generates

```
foobar/
├── .devcontainer/
│   ├── devcontainer.json      # VS Code devcontainer config
│   ├── Dockerfile             # Ubuntu 24.04 base + your chosen tools
│   ├── install-ai-tools.sh   # AI coding tool installer
│   ├── init-firewall.sh      # Allowlist-based outbound firewall
│   └── bashrc.sh             # Shell aliases and environment
└── .gitignore
```

All generated files are scoped to exactly what you selected — nothing more.

## Wizard steps

| Step | What it asks | Pre-selected |
|------|-------------|--------------|
| 1 | Languages & runtimes (Swift, Rust, Python, Node.js/Bun, Go, Ruby, Java/JVM) | — |
| 2 | Databases (PostgreSQL, MySQL/MariaDB, SQLite, Redis, MongoDB) | — |
| 3 | AI coding tools (Claude Code, pi, GitHub Copilot, OpenCode) | Claude Code |
| 4 | Extra CLI tools (ripgrep, fd-find, jq, trash-cli, bat, htop, httpie, just, watchexec) | ripgrep, fd-find, jq, trash-cli, bat |
| 5 | Gitignore patterns | .env, .env.local, *.log, .DS_Store, temp files, dist//build/ |
| 6 | Summary — review and confirm | — |

## Keys

| Key | Action |
|-----|--------|
| `↑` / `↓` or `k` / `j` | Move cursor |
| `Space` | Toggle selection |
| `a` | Select all items in current step |
| `A` | Deselect all items in current step |
| `Home` / `End` | Jump to first/last item |
| `Enter` / `→` | Next step |
| `←` / `b` / `Esc` | Previous step |
| `q` / `Ctrl+C` | Quit |

## Install

```bash
git clone <repo-url>
cd newt
./install.sh
```

Installs the binary to `~/.local/bin/newt`. Pass a different directory as the first argument:

```bash
./install.sh /usr/local/bin
```

### Requirements

- [Rust](https://rustup.rs) (1.70+)

## Usage

```bash
newt <project-name>
```

Opens the wizard in your terminal. On confirmation, the project directory and all files are written to your current working directory.

### Options

| Option | Description |
|--------|-------------|
| `-o, --output <DIR>` | Output directory (default: current directory) |
| `-v, --version` | Print version information |
| `-h, --help` | Print help information |

### Examples

```bash
# Create a new project in the current directory
newt my-project

# Create a project in a specific directory
newt my-project -o ~/projects

# Print version
newt --version
```

## Features

- **Modern terminal UI** — Built with [ratatui](https://ratatui.rs/) for a smooth experience
- **Smart firewall** — Outbound firewall restricts traffic to known domains only
- **Multiple languages** — Swift, Rust, Python, Node.js/Bun, Go, Ruby, Java/JVM
- **Database support** — PostgreSQL, MySQL/MariaDB, SQLite, Redis, MongoDB
- **AI tool integration** — Claude Code, pi, GitHub Copilot, OpenCode
- **Useful CLI tools** — ripgrep, fd-find, jq, trash-cli, bat, htop, httpie, just, watchexec
- **Smart defaults** — Pre-selects commonly used tools for convenience

## Project Structure

```
newt/
├── src/
│   ├── main.rs           # Entry point and CLI
│   ├── app/              # Wizard state and logic
│   │   ├── mod.rs
│   │   ├── state.rs      # App state
│   │   ├── types.rs      # Type definitions
│   │   └── wizard.rs     # Event loop
│   ├── generator/        # File generation
│   │   ├── mod.rs
│   │   ├── config.rs     # Configuration types
│   │   └── templates.rs  # Template rendering
│   └── ui.rs             # Terminal UI rendering
├── Cargo.toml
├── install.sh
└── README.md
```

## License

MIT