//! Template generators for devcontainer files.

use super::config::{AiInstall, Config};

/// Generate devcontainer.json content.
pub fn devcontainer_json(config: &Config) -> String {
    let mut features: Vec<String> = Vec::new();

    // GitHub CLI for Copilot
    if config.ai_tools.needs_github_cli() {
        features.push(r#"    "ghcr.io/devcontainers/features/github-cli:1": {}"#.into());
    }

    // Node.js for pi or Copilot
    if config.ai_tools.needs_node() {
        features.push(r#"    "ghcr.io/devcontainers/features/node:1": { "version": "lts" }"#.into());
    }

    // Common utils (always needed)
    features.push(
        r#"    "ghcr.io/devcontainers/features/common-utils": {
      "installZsh": true,
      "upgradePackages": true
    }"#
        .into(),
    );

    let features_str = features.join(",\n");

    format!(
        r#"{{
  "name": "{}",
  "build": {{
    "dockerfile": "Dockerfile"
  }},
  "features": {{
{}
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
"#,
        config.project_name, features_str
    )
}

/// Generate Dockerfile content.
pub fn dockerfile(config: &Config) -> String {
    // Base packages
    let mut apt_packages = vec![
        "iptables",
        "ipset",
        "iproute2",
        "dnsutils",
        "aggregate",
        "build-essential",
    ];

    // Database packages
    apt_packages.extend(config.databases.apt_packages());

    // Language packages
    apt_packages.extend(config.languages.apt_packages());

    // Extra tool packages
    apt_packages.extend(config.extra_tools.apt_packages());

    let apt_list = apt_packages
        .iter()
        .map(|p| format!("    {}", p))
        .collect::<Vec<_>>()
        .join(" \\\n");

    // fd symlink if needed
    let fd_symlink = if config.extra_tools.needs_fd_symlink() {
        " \\\n    && ln -s $(which fdfind) /usr/local/bin/fd"
    } else {
        ""
    };

    // Collect all user installs
    let mut all_user_installs: Vec<String> = Vec::new();

    for install in config.languages.user_installs() {
        all_user_installs.push(format!("# {}\nRUN {}", install.comment, install.cmd));
    }

    for install in config.extra_tools.user_installs() {
        all_user_installs.push(format!("# {}\nRUN {}", install.comment, install.cmd));
    }

    let user_section = if all_user_installs.is_empty() {
        String::new()
    } else {
        format!("\n{}", all_user_installs.join("\n\n"))
    };

    let ubuntu_img = match config.languages.ubuntu_version() {
        "ubuntu-22.04" => "ubuntu-22.04",
        "ubuntu-20.04" => "ubuntu-20.04",
        _ => "ubuntu-22.04",
    };

    let nim_install = if config.languages.nim {
        r#"
# Nim (arch-aware binary install, works on amd64 and arm64)
RUN NIM_VER=$(curl -sSf https://nim-lang.org/choosenim/stable) \
    && ARCH=$(uname -m) \
    && if [ "$ARCH" = "aarch64" ]; then NIM_ARCH="linux_arm64"; else NIM_ARCH="linux_x64"; fi \
    && curl -LO "https://nim-lang.org/download/nim-${NIM_VER}-${NIM_ARCH}.tar.xz" \
    && tar xf "nim-${NIM_VER}-${NIM_ARCH}.tar.xz" \
    && "nim-${NIM_VER}/install.sh" /usr/local \
    && rm -rf "nim-${NIM_VER}" "nim-${NIM_VER}-${NIM_ARCH}.tar.xz"
"#.to_string()
    } else {
        String::new()
    };

    format!(
        r#"FROM mcr.microsoft.com/devcontainers/base:{}

USER root

RUN apt-get update && apt-get install -y --no-install-recommends \
{} \
    && rm -rf /var/lib/apt/lists/*{}
{}
RUN git config --system --add safe.directory /workspace

COPY install-ai-tools.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/install-ai-tools.sh

COPY init-firewall.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/init-firewall.sh && \
  echo "vscode ALL=(root) NOPASSWD: /usr/local/bin/init-firewall.sh" > /etc/sudoers.d/vscode-firewall && \
  chmod 0440 /etc/sudoers.d/vscode-firewall

COPY bashrc.sh /usr/local/bin/devcontainer-bashrc.sh
RUN echo "source /usr/local/bin/devcontainer-bashrc.sh" >> /home/vscode/.bashrc

USER vscode{}
"#
        ,
        ubuntu_img, apt_list, fd_symlink, nim_install, user_section
    )
}

/// Generate AI tools installer script.
pub fn install_ai_tools(config: &Config) -> String {
    let installs: Vec<AiInstall> = config.ai_tools.install_commands();

    let install_blocks: Vec<String> = installs
        .into_iter()
        .map(|i| {
            format!(
                r#"install_if_missing "{}" "{}" \
    "{}""#,
                i.name, i.check, i.cmd
            )
        })
        .collect();

    let installs_str = if install_blocks.is_empty() {
        String::new()
    } else {
        format!("{}\n\n", install_blocks.join("\n\n"))
    };

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

{}echo "[evanflow] installing skills..."
claude plugin marketplace add evanklem/evanflow 2>/dev/null || \
  claude plugin marketplace update evanflow
claude plugin install evanflow@evanflow 2>/dev/null || true
echo "[evanflow] done"

echo "AI tools ready"
"#,
        installs_str
    )
}

/// Generate firewall initialization script.
pub fn init_firewall(config: &Config) -> String {
    // Collect all required domains
    let mut all_domains: Vec<&str> = Vec::new();
    all_domains.extend(config.languages.domains());
    all_domains.extend(config.ai_tools.domains());

    all_domains.sort_unstable();
    all_domains.dedup();

    let domain_section = if all_domains.is_empty() {
        String::new()
    } else {
        let domain_list = all_domains
            .iter()
            .map(|d| format!("    \"{}\" \\", d))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"
# Resolve and add project-specific allowed domains
for domain in \
{}
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
"#,
            domain_list
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
{}
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
"#,
        domain_section
    )
}

/// Generate bashrc additions.
pub fn bashrc(config: &Config) -> String {
    let mut lines = vec![
        "#!/bin/bash".into(),
        "# Devcontainer shell environment".into(),
        String::new(),
        "alias g='git'".into(),
        "alias vim='TERM=xterm-256color vim'".into(),
    ];

    // Python aliases
    if config.languages.python {
        lines.push(String::new());
        lines.push("alias pm='uv run python manage.py'".into());
        lines.push("alias pmr='uv run python manage.py runserver'".into());
        lines.push("alias pms='uv run python manage.py shell'".into());
    }

    // Rust env
    if config.languages.rust {
        lines.push(String::new());
        lines.push(r#"source "$HOME/.cargo/env""#.into());
    }

    // Go PATH
    if config.languages.go {
        lines.push(String::new());
        lines.push(r#"export PATH="$PATH:/usr/local/go/bin""#.into());
    }

    // Ruby init
    if config.languages.ruby {
        lines.push(String::new());
        lines.push(r#"eval "$(rbenv init -)""#.into());
    }

    // Swift via swiftly
    if config.languages.swift {
        lines.push(String::new());
        lines.push(r#"[ -f "$HOME/.local/share/swiftly/env.sh" ] && source "$HOME/.local/share/swiftly/env.sh""#.into());
    }

    // Timezone
    lines.push(String::new());
    lines.push("export TZ='America/Chicago'".into());

    lines.join("\n") + "\n"
}

/// Generate .gitignore content.
pub fn gitignore(config: &Config) -> String {
    let entries = config.gitignore.entries(&config.languages);
    entries.join("\n") + "\n"
}