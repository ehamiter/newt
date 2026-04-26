//! Devcontainer file generation.

pub mod config;
pub mod templates;

use config::Config;
use std::fs;
use std::path::Path;

/// Generate all devcontainer files for a new project.
pub fn generate(path: &Path, answers: &crate::app::Answers) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from(answers.clone());

    let dc = path.join(".devcontainer");
    fs::create_dir_all(&dc)?;

    fs::write(dc.join("devcontainer.json"), templates::devcontainer_json(&config))?;
    fs::write(dc.join("Dockerfile"), templates::dockerfile(&config))?;
    fs::write(
        dc.join("install-ai-tools.sh"),
        templates::install_ai_tools(&config),
    )?;
    fs::write(dc.join("init-firewall.sh"), templates::init_firewall(&config))?;
    fs::write(dc.join("bashrc.sh"), templates::bashrc(&config))?;
    fs::write(path.join(".gitignore"), templates::gitignore(&config))?;

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

#[cfg(test)]
mod tests {
    use super::config::{AiToolSet, Config, DatabaseSet, ExtraToolSet, GitignoreSet, LanguageSet};
    use super::templates;

    fn test_config() -> Config {
        Config {
            project_name: "test-project".into(),
            languages: LanguageSet {
                rust: true,
                python: true,
                node_bun: false,
                go: false,
                ruby: false,
                java_jvm: false,
            },
            databases: DatabaseSet {
                postgresql: true,
                ..Default::default()
            },
            ai_tools: AiToolSet {
                claude_code: true,
                ..Default::default()
            },
            extra_tools: ExtraToolSet {
                ripgrep: true,
                ..Default::default()
            },
            gitignore: GitignoreSet {
                env: true,
                target: true,
                ..Default::default()
            },
        }
    }

    #[test]
    fn test_devcontainer_json_has_name() {
        let config = test_config();
        let json = templates::devcontainer_json(&config);
        assert!(json.contains(r#""name": "test-project""#));
    }

    #[test]
    fn test_dockerfile_has_apt_packages() {
        let config = test_config();
        let dockerfile = templates::dockerfile(&config);
        assert!(dockerfile.contains("ripgrep"));
        assert!(dockerfile.contains("libpq-dev"));
    }

    #[test]
    fn test_install_ai_tools_has_claude() {
        let config = test_config();
        let script = templates::install_ai_tools(&config);
        assert!(script.contains("claude"));
        assert!(script.contains("claude.ai/install.sh"));
    }

    #[test]
    fn test_gitignore_has_rust_target() {
        let config = test_config();
        let gitignore = templates::gitignore(&config);
        assert!(gitignore.contains("target/"));
    }
}