//! Configuration types for file generation.

use crate::app::Answers;

/// Configuration derived from wizard answers.
#[derive(Debug, Clone)]
pub struct Config {
    pub project_name: String,
    pub languages: LanguageSet,
    pub databases: DatabaseSet,
    pub ai_tools: AiToolSet,
    pub extra_tools: ExtraToolSet,
    pub gitignore: GitignoreSet,
}

impl From<Answers> for Config {
    fn from(answers: Answers) -> Self {
        Self {
            project_name: answers.project_name,
            languages: LanguageSet::from_labels(&answers.languages),
            databases: DatabaseSet::from_labels(&answers.databases),
            ai_tools: AiToolSet::from_labels(&answers.ai_tools),
            extra_tools: ExtraToolSet::from_labels(&answers.extra_tools),
            gitignore: GitignoreSet::from_labels(&answers.gitignore),
        }
    }
}

/// Language/runtime selections.
#[derive(Debug, Clone, Default)]
pub struct LanguageSet {
    pub rust: bool,
    pub python: bool,
    pub node_bun: bool,
    pub go: bool,
    pub ruby: bool,
    pub java_jvm: bool,
    pub swift: bool,
    /// Ubuntu version for devcontainer (default: 24.04)
    pub ubuntu_version: Option<String>,
}

impl LanguageSet {
    fn from_labels(labels: &[&str]) -> Self {
        Self {
            rust: labels.contains(&"Rust"),
            python: labels.contains(&"Python"),
            node_bun: labels.contains(&"Node.js / Bun"),
            go: labels.contains(&"Go"),
            ruby: labels.contains(&"Ruby"),
            java_jvm: labels.contains(&"Java / JVM"),
            swift: labels.contains(&"Swift"),
            ubuntu_version: None,
        }
    }

    /// Get the Ubuntu version for the devcontainer.
    pub fn ubuntu_version(&self) -> &str {
        self.ubuntu_version.as_deref().unwrap_or("ubuntu-22.04")
    }

    /// Get all required apt packages for selected languages.
    pub fn apt_packages(&self) -> Vec<&'static str> {
        let mut packages = Vec::new();
        if self.java_jvm {
            packages.extend(["default-jdk", "maven"]);
        }
        if self.swift {
            packages.extend([
                "binutils-gold",
                "libcurl4-openssl-dev",
                "libicu-dev",
                "libncurses-dev",
                "libsqlite3-dev",
                "libxml2-dev",
                "uuid-dev",
            ]);
        }
        packages
    }

    /// Get all user-level install commands.
    pub fn user_installs(&self) -> Vec<UserInstall> {
        let mut installs = Vec::new();

        if self.rust {
            installs.push(UserInstall {
                cmd: "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y"
                    .into(),
                comment: "Rust toolchain".into(),
            });
        }

        if self.python {
            installs.push(UserInstall {
                cmd: "curl -LsSf https://astral.sh/uv/install.sh | sh \\\n    && /home/vscode/.local/bin/uv python install 3.13"
                    .into(),
                comment: "Python via uv".into(),
            });
        }

        if self.node_bun {
            installs.push(UserInstall {
                cmd: "curl -fsSL https://bun.sh/install | bash".into(),
                comment: "Bun runtime".into(),
            });
        }

        if self.go {
            // Use latest stable Go version
            installs.push(UserInstall {
                cmd: "GO_VERSION=$(curl -s https://go.dev/VERSION?m=text) \\\n    && curl -LO \"https://go.dev/dl/${GO_VERSION}.linux-amd64.tar.gz\" \\\n    && sudo tar -C /usr/local -xzf ${GO_VERSION}.linux-amd64.tar.gz \\\n    && rm ${GO_VERSION}.linux-amd64.tar.gz"
                    .into(),
                comment: "Go toolchain".into(),
            });
        }

        if self.ruby {
            installs.push(UserInstall {
                cmd: "curl -fsSL https://rbenv.org/install.sh | bash \\\n    && rbenv install 3.3.0 \\\n    && rbenv global 3.3.0"
                    .into(),
                comment: "Ruby via rbenv".into(),
            });
        }

        if self.swift {
            installs.push(UserInstall {
                cmd: "cd /tmp && curl -O https://download.swift.org/swiftly/linux/swiftly-$(uname -m).tar.gz \\\n    && tar zxf swiftly-$(uname -m).tar.gz \\\n    && ./swiftly init --quiet-shell-followup \\\n    && $HOME/.local/share/swiftly/bin/swiftly install latest"
                    .into(),
                comment: "Swift via swiftly".into(),
            });
        }

        installs
    }

    /// Get domains needed for these languages.
    pub fn domains(&self) -> Vec<&'static str> {
        let mut domains = Vec::new();
        if self.rust {
            domains.extend([
                "sh.rustup.rs",
                "static.rust-lang.org",
                "crates.io",
                "static.crates.io",
                "index.crates.io",
            ]);
        }
        if self.python {
            domains.extend(["astral.sh", "pypi.org", "files.pythonhosted.org"]);
        }
        if self.node_bun {
            domains.extend(["registry.npmjs.org", "bun.sh"]);
        }
        if self.swift {
            domains.extend([
                "download.swift.org",
                "github.com",
            ]);
        }
        domains.sort_unstable();
        domains.dedup();
        domains
    }

    /// Check if bashrc needs PATH modifications.
    #[allow(dead_code)]
    pub fn needs_bashrc_extras(&self) -> bool {
        self.rust || self.go || self.ruby || self.swift
    }
}

/// A user-level installation command.
#[derive(Debug, Clone)]
pub struct UserInstall {
    pub cmd: String,
    pub comment: String,
}

/// Database selections.
#[derive(Debug, Clone, Default)]
pub struct DatabaseSet {
    pub postgresql: bool,
    pub mysql: bool,
    pub sqlite: bool,
    pub redis: bool,
    #[allow(dead_code)]
    pub mongodb: bool,
}

impl DatabaseSet {
    fn from_labels(labels: &[&str]) -> Self {
        Self {
            postgresql: labels.contains(&"PostgreSQL"),
            mysql: labels.contains(&"MySQL / MariaDB"),
            sqlite: labels.contains(&"SQLite"),
            redis: labels.contains(&"Redis"),
            mongodb: labels.contains(&"MongoDB"),
        }
    }

    /// Get required apt packages.
    pub fn apt_packages(&self) -> Vec<&'static str> {
        let mut packages = Vec::new();
        if self.postgresql {
            packages.extend(["libpq-dev", "postgresql-client"]);
        }
        if self.mysql {
            packages.extend(["default-libmysqlclient-dev", "default-mysql-client"]);
        }
        if self.sqlite {
            packages.extend(["libsqlite3-dev", "sqlite3"]);
        }
        if self.redis {
            packages.push("redis-tools");
        }
        packages
    }
}

/// AI tool selections.
#[derive(Debug, Clone, Default)]
pub struct AiToolSet {
    pub claude_code: bool,
    pub copilot: bool,
    pub opencode: bool,
    pub pi: bool,
}

impl AiToolSet {
    fn from_labels(labels: &[&str]) -> Self {
        Self {
            claude_code: labels.contains(&"Claude Code"),
            copilot: labels.contains(&"GitHub Copilot"),
            opencode: labels.contains(&"OpenCode"),
            pi: labels.contains(&"pi"),
        }
    }

    /// Check if Node.js is needed (for Copilot or pi).
    pub fn needs_node(&self) -> bool {
        self.copilot || self.pi
    }

    /// Check if GitHub CLI is needed.
    pub fn needs_github_cli(&self) -> bool {
        self.copilot
    }

    /// Get install commands for AI tools.
    pub fn install_commands(&self) -> Vec<AiInstall> {
        let mut installs = Vec::new();

        if self.claude_code {
            installs.push(AiInstall {
                name: "claude",
                check: "claude",
                cmd: "curl -fsSL https://claude.ai/install.sh | bash",
            });
        }

        if self.copilot {
            installs.push(AiInstall {
                name: "copilot",
                check: "gh copilot",
                cmd: "gh extension install github/gh-copilot",
            });
        }

        if self.opencode {
            installs.push(AiInstall {
                name: "opencode",
                check: "opencode",
                cmd: "curl -fsSL https://opencode.ai/install | bash",
            });
        }

        if self.pi {
            installs.push(AiInstall {
                name: "pi",
                check: "pi",
                cmd: "npm install -g @mariozechner/pi-coding-agent",
            });
        }

        installs
    }

    /// Get agent identifiers for the skills CLI `-a` flag.
    pub fn skills_agents(&self) -> Vec<&'static str> {
        let mut agents = Vec::new();
        if self.claude_code {
            agents.push("claude-code");
        }
        if self.copilot {
            agents.push("github-copilot");
        }
        if self.opencode {
            agents.push("opencode");
        }
        if self.pi {
            agents.push("pi");
        }
        agents
    }

    /// Get required domains for firewall configuration.
    pub fn domains(&self) -> Vec<&'static str> {
        let mut domains = Vec::new();
        if self.claude_code {
            domains.extend([
                "api.anthropic.com",
                "claude.ai",
                "downloads.claude.ai",
                "statsig.anthropic.com",
            ]);
        }
        if self.copilot {
            domains.extend([
                "api.githubcopilot.com",
                "copilot-proxy.githubusercontent.com",
            ]);
        }
        if self.opencode {
            domains.push("opencode.ai");
        }
        if self.pi {
            domains.push("registry.npmjs.org");
        }
        domains.sort_unstable();
        domains.dedup();
        domains
    }
}

/// An AI tool installation.
#[derive(Debug, Clone)]
pub struct AiInstall {
    pub name: &'static str,
    pub check: &'static str,
    pub cmd: &'static str,
}

/// Extra CLI tool selections.
#[derive(Debug, Clone, Default)]
pub struct ExtraToolSet {
    pub ripgrep: bool,
    pub fd_find: bool,
    #[allow(dead_code)]
    pub jq: bool,
    pub trash_cli: bool,
    pub bat: bool,
    pub htop: bool,
    pub httpie: bool,
    pub just: bool,
    pub watchexec: bool,
}

impl ExtraToolSet {
    fn from_labels(labels: &[&str]) -> Self {
        Self {
            ripgrep: labels.contains(&"ripgrep"),
            fd_find: labels.contains(&"fd-find"),
            jq: labels.contains(&"jq"),
            trash_cli: labels.contains(&"trash-cli"),
            bat: labels.contains(&"bat"),
            htop: labels.contains(&"htop"),
            httpie: labels.contains(&"httpie"),
            just: labels.contains(&"just"),
            watchexec: labels.contains(&"watchexec"),
        }
    }

    /// Get apt packages needed.
    pub fn apt_packages(&self) -> Vec<&'static str> {
        let mut packages = vec!["jq"];
        if self.ripgrep {
            packages.push("ripgrep");
        }
        if self.fd_find {
            packages.push("fd-find");
        }
        if self.trash_cli {
            packages.push("trash-cli");
        }
        if self.bat {
            packages.push("bat");
        }
        if self.htop {
            packages.push("htop");
        }
        if self.httpie {
            packages.push("httpie");
        }
        packages
    }

    /// Check if fd-find symlink is needed.
    pub fn needs_fd_symlink(&self) -> bool {
        self.fd_find
    }

    /// Get user-level installs (non-apt tools).
    pub fn user_installs(&self) -> Vec<UserInstall> {
        let mut installs = Vec::new();

        if self.just {
            installs.push(UserInstall {
                cmd: "curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash -s -- --to /home/vscode/.local/bin"
                    .into(),
                comment: "just command runner".into(),
            });
        }

        if self.watchexec {
            // Use cargo-binstall for reliable binary installation
            installs.push(UserInstall {
                cmd: "curl -L https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash \\\n    && cargo binstall watchexec-cli -y"
                    .into(),
                comment: "watchexec file watcher".into(),
            });
        }

        installs
    }
}

/// Gitignore pattern selections.
#[derive(Debug, Clone, Default)]
pub struct GitignoreSet {
    pub env: bool,
    pub env_local: bool,
    pub logs: bool,
    pub ds_store: bool,
    pub temp_files: bool,
    pub target: bool,
    pub node_modules: bool,
    pub venv: bool,
    pub pycache: bool,
    pub build: bool,
    pub ide: bool,
}

impl GitignoreSet {
    fn from_labels(labels: &[&str]) -> Self {
        Self {
            env: labels.contains(&".env"),
            env_local: labels.contains(&".env.local"),
            logs: labels.contains(&"*.log"),
            ds_store: labels.contains(&".DS_Store"),
            temp_files: labels.contains(&"*.tmp / *.swp"),
            target: labels.contains(&"target/"),
            node_modules: labels.contains(&"node_modules/"),
            venv: labels.contains(&".venv/ / venv/"),
            pycache: labels.contains(&"__pycache__/ / *.pyc"),
            build: labels.contains(&"dist/ / build/"),
            ide: labels.contains(&".idea/ / .vscode/"),
        }
    }

    /// Expand to full gitignore entries.
    pub fn entries(&self, langs: &LanguageSet) -> Vec<String> {
        let mut entries = Vec::new();

        if self.env {
            entries.push(".env".into());
        }
        if self.env_local {
            entries.push(".env.local".into());
            entries.push(".env.*.local".into());
        }
        if self.logs {
            entries.push("*.log".into());
            entries.push("logs/".into());
        }
        if self.ds_store {
            entries.push(".DS_Store".into());
            entries.push("._*".into());
        }
        if self.temp_files {
            entries.push("*.tmp".into());
            entries.push("*.swp".into());
            entries.push("*~".into());
        }
        if self.target || langs.rust {
            entries.push("target/".into());
        }
        if self.node_modules || langs.node_bun {
            entries.push("node_modules/".into());
            entries.push(".bun/".into());
        }
        if self.venv || langs.python {
            entries.push(".venv/".into());
            entries.push("venv/".into());
        }
        if self.pycache || langs.python {
            entries.push("__pycache__/".into());
            entries.push("*.pyc".into());
            entries.push("*.pyo".into());
        }
        if langs.go {
            entries.push("*.exe".into());
            entries.push("*.test".into());
        }
        if langs.swift {
            entries.push(".build/".into());
            entries.push("*.o".into());
            entries.push("*.d".into());
        }
        if self.build {
            entries.push("dist/".into());
            entries.push("build/".into());
        }
        if self.ide {
            entries.push(".idea/".into());
            entries.push(".vscode/".into());
        }

        entries.sort();
        entries.dedup();
        entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_set_from_labels() {
        let langs = LanguageSet::from_labels(&["Rust", "Python"]);
        assert!(langs.rust);
        assert!(langs.python);
        assert!(!langs.go);
    }

    #[test]
    fn test_database_apt_packages() {
        let db = DatabaseSet::from_labels(&["PostgreSQL", "SQLite"]);
        let pkgs = db.apt_packages();
        assert!(pkgs.contains(&"libpq-dev"));
        assert!(pkgs.contains(&"libsqlite3-dev"));
    }

    #[test]
    fn test_ai_tool_domains() {
        let ai = AiToolSet::from_labels(&["Claude Code", "GitHub Copilot"]);
        let domains = ai.domains();
        assert!(domains.contains(&"api.anthropic.com"));
        assert!(domains.contains(&"api.githubcopilot.com"));
    }

    #[test]
    fn test_gitignore_entries() {
        let gi = GitignoreSet::from_labels(&[".env", "*.log"]);
        let langs = LanguageSet::default();
        let entries = gi.entries(&langs);
        assert!(entries.contains(&".env".into()));
        assert!(entries.contains(&"*.log".into()));
    }
}