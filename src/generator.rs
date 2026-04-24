use std::{fs, path::Path};

use crate::app::Answers;

pub fn generate(path: &Path, answers: &Answers) -> Result<(), Box<dyn std::error::Error>> {
    let dc = path.join(".devcontainer");
    fs::create_dir_all(&dc)?;

    fs::write(dc.join("devcontainer.json"), devcontainer_json(answers))?;
    fs::write(dc.join("Dockerfile"), dockerfile(answers))?;
    fs::write(dc.join("install-ai-tools.sh"), install_ai_tools(answers))?;
    fs::write(dc.join("init-firewall.sh"), init_firewall(answers))?;
    fs::write(dc.join("bashrc.sh"), bashrc(answers))?;
    fs::write(path.join(".gitignore"), gitignore(answers))?;

    make_executable(&dc.join("install-ai-tools.sh"))?;
    make_executable(&dc.join("init-firewall.sh"))?;

    Ok(())
}

#[cfg(unix)]
fn make_executable(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms)?;
    Ok(())
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

fn has(list: &[&str], label: &str) -> bool {
    list.contains(&label)
}

fn devcontainer_json(answers: &Answers) -> String {
    let mut features: Vec<String> = Vec::new();

    if has(&answers.ai_tools, "GitHub Copilot") || has(&answers.languages, "Node.js / Bun") {
        features.push(
            r#"    "ghcr.io/devcontainers/features/github-cli:1": {}"#.to_string(),
        );
    }

    if has(&answers.languages, "Node.js / Bun") {
        features.push(
            r#"    "ghcr.io/devcontainers/features/node:1": { "version": "lts" }"#.to_string(),
        );
    }

    features.push(
        r#"    "ghcr.io/devcontainers/features/common-utils:2": {
      "installZsh": true,
      "upgradePackages": true
    }"#
        .to_string(),
    );

    let features_str = features.join(",\n");
    let name = &answers.project_name;

    format!(
        r#"{{
  "name": "{name}",
  "build": {{
    "dockerfile": "Dockerfile"
  }},
  "features": {{
{features_str}
  }},
  "runArgs": [
    "--cap-add=NET_ADMIN",
    "--cap-add=NET_RAW"
  ],
  "workspaceMount": "source=${{localWorkspaceFolder}},target=/workspace,type=bind,consistency=cached",
  "workspaceFolder": "/workspace",
  "initializeCommand": "mkdir -p /tmp/clipbridge $HOME/.claude $HOME/.config/gh",
  "mounts": [
    "source=/tmp/clipbridge,target=/clipboard,type=bind,readonly",
    "source=${{localEnv:HOME}}/.claude,target=/home/vscode/.claude,type=bind"
  ],
  "remoteEnv": {{
    "PATH": "/home/vscode/.local/bin:${{containerEnv:PATH}}"
  }},
  "remoteUser": "vscode",
  "postStartCommand": "sudo /usr/local/bin/init-firewall.sh && /usr/local/bin/install-ai-tools.sh",
  "waitFor": "postStartCommand"
}}
"#
    )
}

fn dockerfile(answers: &Answers) -> String {
    let mut apt_packages: Vec<String> = vec![
        "iptables".into(),
        "ipset".into(),
        "iproute2".into(),
        "dnsutils".into(),
        "aggregate".into(),
        "build-essential".into(),
    ];

    // Databases
    if has(&answers.databases, "PostgreSQL") {
        apt_packages.push("libpq-dev".into());
        apt_packages.push("postgresql-client".into());
    }
    if has(&answers.databases, "MySQL / MariaDB") {
        apt_packages.push("default-libmysqlclient-dev".into());
        apt_packages.push("default-mysql-client".into());
    }
    if has(&answers.databases, "SQLite") {
        apt_packages.push("libsqlite3-dev".into());
        apt_packages.push("sqlite3".into());
    }
    if has(&answers.databases, "Redis") {
        apt_packages.push("redis-tools".into());
    }

    // Languages installed via apt (Java)
    if has(&answers.languages, "Java / JVM") {
        apt_packages.push("default-jdk".into());
        apt_packages.push("maven".into());
    }

    // Extra tools via apt
    let mut need_fd_symlink = false;
    for tool in &answers.extra_tools {
        match *tool {
            "ripgrep" => apt_packages.push("ripgrep".into()),
            "fd-find" => {
                apt_packages.push("fd-find".into());
                need_fd_symlink = true;
            }
            "jq" => apt_packages.push("jq".into()),
            "trash-cli" => apt_packages.push("trash-cli".into()),
            "bat" => apt_packages.push("bat".into()),
            "htop" => apt_packages.push("htop".into()),
            "httpie" => apt_packages.push("httpie".into()),
            _ => {}
        }
    }

    let apt_list = apt_packages
        .iter()
        .map(|p| format!("    {}", p))
        .collect::<Vec<_>>()
        .join(" \\\n");

    let fd_symlink = if need_fd_symlink {
        "\n    && ln -s $(which fdfind) /usr/local/bin/fd"
    } else {
        ""
    };

    // User-level installs
    let mut user_installs: Vec<String> = Vec::new();

    if has(&answers.languages, "Rust") {
        user_installs.push(
            "RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y"
                .to_string(),
        );
    }
    if has(&answers.languages, "Python") {
        user_installs.push(
            "RUN curl -LsSf https://astral.sh/uv/install.sh | sh \\\n    && /home/vscode/.local/bin/uv python install 3.13"
                .to_string(),
        );
    }
    if has(&answers.languages, "Node.js / Bun") {
        user_installs.push("RUN curl -fsSL https://bun.sh/install | bash".to_string());
    }
    if has(&answers.languages, "Go") {
        user_installs.push(
            "RUN curl -LO https://go.dev/dl/go1.23.4.linux-amd64.tar.gz \\\n    && sudo tar -C /usr/local -xzf go1.23.4.linux-amd64.tar.gz \\\n    && rm go1.23.4.linux-amd64.tar.gz"
                .to_string(),
        );
    }
    if has(&answers.languages, "Ruby") {
        user_installs.push(
            "RUN curl -fsSL https://rbenv.org/install.sh | bash \\\n    && rbenv install 3.3.0 \\\n    && rbenv global 3.3.0"
                .to_string(),
        );
    }

    // Extra tools installed via binary
    if has(&answers.extra_tools, "delta") {
        user_installs.push(
            "RUN curl -LO https://github.com/dandavison/delta/releases/latest/download/git-delta_0.17.0_amd64.deb \\\n    && sudo dpkg -i git-delta_0.17.0_amd64.deb \\\n    && rm git-delta_0.17.0_amd64.deb"
                .to_string(),
        );
    }
    if has(&answers.extra_tools, "just") {
        user_installs.push(
            "RUN curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash -s -- --to /home/vscode/.local/bin"
                .to_string(),
        );
    }
    if has(&answers.extra_tools, "watchexec") {
        user_installs.push(
            "RUN curl -LO https://github.com/watchexec/watchexec/releases/latest/download/watchexec-2.1.2-x86_64-unknown-linux-musl.tar.xz \\\n    && tar xf watchexec-*.tar.xz \\\n    && mv watchexec-*/watchexec /home/vscode/.local/bin/ \\\n    && rm -rf watchexec-*"
                .to_string(),
        );
    }

    let user_section = if user_installs.is_empty() {
        String::new()
    } else {
        format!("\n{}", user_installs.join("\n\n"))
    };

    format!(
        r#"FROM mcr.microsoft.com/devcontainers/base:ubuntu-24.04

USER root

RUN apt-get update && apt-get install -y --no-install-recommends \
{apt_list} \
    && rm -rf /var/lib/apt/lists/*{fd_symlink}

RUN git config --system --add safe.directory /workspace

COPY install-ai-tools.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/install-ai-tools.sh

COPY init-firewall.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/init-firewall.sh && \
  echo "vscode ALL=(root) NOPASSWD: /usr/local/bin/init-firewall.sh" > /etc/sudoers.d/vscode-firewall && \
  chmod 0440 /etc/sudoers.d/vscode-firewall

USER vscode
{user_section}
"#
    )
}

fn install_ai_tools(answers: &Answers) -> String {
    let mut installs: Vec<String> = Vec::new();

    if has(&answers.ai_tools, "Claude Code") {
        installs.push(
            r#"install_if_missing "claude" "claude" \
    "curl -fsSL https://claude.ai/install.sh | bash""#
                .to_string(),
        );
    }
    if has(&answers.ai_tools, "GitHub Copilot") {
        installs.push(
            r#"install_if_missing "copilot" "gh copilot" \
    "gh extension install github/gh-copilot""#
                .to_string(),
        );
    }
    if has(&answers.ai_tools, "OpenCode") {
        installs.push(
            r#"install_if_missing "opencode" "opencode" \
    "curl -fsSL https://opencode.ai/install | bash""#
                .to_string(),
        );
    }

    let installs_str = installs.join("\n\n");

    format!(
        r#"#!/bin/bash
set -e

install_if_missing() {{
    local name=$1
    local check_cmd=$2
    local install_cmd=$3

    if command -v "$check_cmd" &>/dev/null; then
        echo "[$name] already installed, skipping"
    else
        echo "[$name] installing..."
        eval "$install_cmd"
        echo "[$name] done"
    fi
}}

{installs_str}

echo "AI tools ready"
"#
    )
}

fn init_firewall(answers: &Answers) -> String {
    // Build the list of allowed domains based on selected tools
    let mut extra_domains: Vec<&str> = Vec::new();

    if has(&answers.languages, "Rust") {
        extra_domains.push("sh.rustup.rs");
        extra_domains.push("static.rust-lang.org");
        extra_domains.push("crates.io");
        extra_domains.push("static.crates.io");
        extra_domains.push("index.crates.io");
    }
    if has(&answers.languages, "Python") {
        extra_domains.push("astral.sh");
        extra_domains.push("pypi.org");
        extra_domains.push("files.pythonhosted.org");
    }
    if has(&answers.languages, "Node.js / Bun") {
        extra_domains.push("registry.npmjs.org");
        extra_domains.push("bun.sh");
    }
    if has(&answers.ai_tools, "Claude Code") {
        extra_domains.push("api.anthropic.com");
        extra_domains.push("claude.ai");
        extra_domains.push("downloads.claude.ai");
        extra_domains.push("statsig.anthropic.com");
    }
    if has(&answers.ai_tools, "GitHub Copilot") {
        extra_domains.push("api.githubcopilot.com");
        extra_domains.push("copilot-proxy.githubusercontent.com");
    }
    if has(&answers.ai_tools, "OpenCode") {
        extra_domains.push("opencode.ai");
    }

    // Deduplicate
    extra_domains.sort_unstable();
    extra_domains.dedup();

    let domain_list = extra_domains
        .iter()
        .map(|d| format!("    \"{}\" \\", d))
        .collect::<Vec<_>>()
        .join("\n");

    let domain_section = if extra_domains.is_empty() {
        String::new()
    } else {
        format!(
            r#"
# Resolve and add project-specific allowed domains
for domain in \
{domain_list}
    ; do
    echo "Resolving $domain..."
    ips=$(dig +noall +answer A "$domain" | awk '$4 == "A" {{print $5}}')
    if [ -z "$ips" ]; then
        echo "WARNING: Failed to resolve $domain, skipping"
        continue
    fi
    while read -r ip; do
        ipset add allowed-domains "$ip" -exist
    done < <(echo "$ips")
done
"#
        )
    };

    format!(
        r#"#!/bin/bash
set -euo pipefail
IFS=$'\n\t'

DOCKER_DNS_RULES=$(iptables-save -t nat | grep "127\.0\.0\.11" || true)

iptables -F
iptables -X
iptables -t nat -F
iptables -t nat -X
iptables -t mangle -F
iptables -t mangle -X
ipset destroy allowed-domains 2>/dev/null || true

iptables -P INPUT ACCEPT
iptables -P OUTPUT ACCEPT
iptables -P FORWARD ACCEPT

if [ -n "$DOCKER_DNS_RULES" ]; then
    iptables -t nat -N DOCKER_OUTPUT 2>/dev/null || true
    iptables -t nat -N DOCKER_POSTROUTING 2>/dev/null || true
    echo "$DOCKER_DNS_RULES" | xargs -L 1 iptables -t nat
fi

iptables -A OUTPUT -p udp --dport 53 -j ACCEPT
iptables -A INPUT -p udp -s 127.0.0.11 --sport 53 -m state --state ESTABLISHED -j ACCEPT
iptables -A INPUT -i lo -j ACCEPT
iptables -A OUTPUT -o lo -j ACCEPT

ipset create allowed-domains hash:net

echo "Fetching GitHub IP ranges..."
gh_ranges=$(curl -sS --fail --connect-timeout 5 --max-time 20 https://api.github.com/meta)
while read -r cidr; do
    [[ "$cidr" =~ ^[0-9]{{1,3}}\.[0-9]{{1,3}}\.[0-9]{{1,3}}\.[0-9]{{1,3}}/[0-9]{{1,2}}$ ]] || continue
    ipset add allowed-domains "$cidr"
done < <(echo "$gh_ranges" | jq -r '(.web + .api + .git)[] | select(contains(":") | not)' | aggregate -q)
{domain_section}
HOST_IP=$(ip route | grep default | cut -d" " -f3)
iptables -A INPUT -s "$HOST_IP" -j ACCEPT
iptables -A OUTPUT -d "$HOST_IP" -j ACCEPT

iptables -A OUTPUT -p tcp --dport 22 -m set --match-set allowed-domains dst -j ACCEPT
iptables -A INPUT -p tcp --sport 22 -m state --state ESTABLISHED -j ACCEPT

iptables -P INPUT DROP
iptables -P FORWARD DROP
iptables -P OUTPUT DROP

iptables -A INPUT -m state --state ESTABLISHED,RELATED -j ACCEPT
iptables -A OUTPUT -m state --state ESTABLISHED,RELATED -j ACCEPT
iptables -A OUTPUT -m set --match-set allowed-domains dst -j ACCEPT
iptables -A OUTPUT -j REJECT --reject-with icmp-admin-prohibited

echo "Firewall configuration complete"
"#
    )
}

fn bashrc(answers: &Answers) -> String {
    let mut lines: Vec<String> = vec![
        "#!/bin/bash".into(),
        "# Devcontainer shell environment".into(),
        String::new(),
        "alias g='git'".into(),
        "alias vim='TERM=xterm-256color vim'".into(),
    ];

    if has(&answers.languages, "Python") {
        lines.push(String::new());
        lines.push("alias pm='uv run python manage.py'".into());
        lines.push("alias pmr='uv run python manage.py runserver'".into());
        lines.push("alias pms='uv run python manage.py shell'".into());
    }

    if has(&answers.languages, "Rust") {
        lines.push(String::new());
        lines.push(r#"source "$HOME/.cargo/env""#.into());
    }

    if has(&answers.languages, "Go") {
        lines.push(String::new());
        lines.push(r#"export PATH="$PATH:/usr/local/go/bin""#.into());
    }

    if has(&answers.languages, "Ruby") {
        lines.push(String::new());
        lines.push(r#"eval "$(rbenv init -)""#.into());
    }

    lines.push(String::new());
    lines.push("export TZ='America/Chicago'".into());

    lines.join("\n") + "\n"
}

fn gitignore(answers: &Answers) -> String {
    let mut entries: Vec<String> = Vec::new();

    for &label in &answers.gitignore {
        match label {
            ".env" => entries.push(".env".into()),
            ".env.local" => {
                entries.push(".env.local".into());
                entries.push(".env.*.local".into());
            }
            "*.log" => {
                entries.push("*.log".into());
                entries.push("logs/".into());
            }
            ".DS_Store" => {
                entries.push(".DS_Store".into());
                entries.push("._*".into());
            }
            "*.tmp / *.swp" => {
                entries.push("*.tmp".into());
                entries.push("*.swp".into());
                entries.push("*~".into());
            }
            "target/" => entries.push("target/".into()),
            "node_modules/" => entries.push("node_modules/".into()),
            ".venv/ / venv/" => {
                entries.push(".venv/".into());
                entries.push("venv/".into());
            }
            "__pycache__/ / *.pyc" => {
                entries.push("__pycache__/".into());
                entries.push("*.pyc".into());
                entries.push("*.pyo".into());
            }
            "dist/ / build/" => {
                entries.push("dist/".into());
                entries.push("build/".into());
            }
            ".idea/ / .vscode/" => {
                entries.push(".idea/".into());
                entries.push(".vscode/".into());
            }
            _ => entries.push(label.into()),
        }
    }

    // Auto-add language-appropriate entries not already present
    let auto = |entries: &mut Vec<String>, entry: &str| {
        let s = entry.to_string();
        if !entries.contains(&s) {
            entries.push(s);
        }
    };

    if has(&answers.languages, "Rust") {
        auto(&mut entries, "target/");
    }
    if has(&answers.languages, "Python") {
        auto(&mut entries, ".venv/");
        auto(&mut entries, "__pycache__/");
        auto(&mut entries, "*.pyc");
    }
    if has(&answers.languages, "Node.js / Bun") {
        auto(&mut entries, "node_modules/");
        auto(&mut entries, ".bun/");
    }
    if has(&answers.languages, "Go") {
        auto(&mut entries, "*.exe");
        auto(&mut entries, "*.test");
    }

    entries.dedup();
    entries.join("\n") + "\n"
}
