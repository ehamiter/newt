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

| Step | What it asks |
|------|-------------|
| 1 | Languages & runtimes (Rust, Python, Node.js/Bun, Go, Ruby, Java) |
| 2 | Databases (PostgreSQL, MySQL, SQLite, Redis, MongoDB) |
| 3 | AI coding tools (Claude Code, GitHub Copilot, OpenCode) |
| 4 | Extra CLI tools (ripgrep, fd, jq, bat, delta, htop, …) |
| 5 | Gitignore patterns |
| 6 | Summary — review and confirm |

## Keys

| Key | Action |
|-----|--------|
| `↑` / `↓` or `k` / `j` | Move cursor |
| `Space` | Toggle selection |
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
