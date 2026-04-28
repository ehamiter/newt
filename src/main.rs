use clap::Parser;
use std::io::{self, Write};
use std::path::PathBuf;

mod app;
mod generator;
mod ui;

#[derive(Parser)]
#[command(name = "newt", version, about = "Scaffold a new project with .devcontainer setup")]
struct Cli {
    /// Project name — a new directory with this name will be created here.
    /// Omit to scaffold into the current directory instead.
    project_name: Option<String>,

    /// Output directory (default: current directory)
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn validate_project_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Project name cannot be empty".into());
    }
    if name.starts_with('.') {
        return Err("Project name cannot start with a dot".into());
    }
    if name.contains('/') || name.contains('\\') {
        return Err("Project name cannot contain path separators".into());
    }
    let invalid_chars = ['\0', '/', '\\'];
    for c in invalid_chars {
        if name.contains(c) {
            return Err(format!("Project name contains invalid character: {}", c));
        }
    }
    if name.chars().any(|c| c.is_whitespace()) {
        return Err("Project name cannot contain whitespace".into());
    }
    Ok(())
}

fn confirm_overwrite(subject: &str) -> bool {
    print!("{} already exists. Overwrite it? [y/N]: ", subject);
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let base_dir = match cli.output {
        Some(p) => p,
        None => std::env::current_dir()?,
    };

    let (project_path, project_name) = match cli.project_name {
        Some(name) => {
            if let Err(e) = validate_project_name(&name) {
                eprintln!("error: {}", e);
                std::process::exit(1);
            }
            let path = base_dir.join(&name);
            (path, name)
        }
        None => {
            let name = base_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("project")
                .to_string();
            (base_dir, name)
        }
    };

    let dc_path = project_path.join(".devcontainer");
    if dc_path.exists() {
        if !confirm_overwrite(".devcontainer/") {
            println!("Aborted.");
            return Ok(());
        }
        std::fs::remove_dir_all(&dc_path)?;
    }

    let gi_path = project_path.join(".gitignore");
    if gi_path.exists() && !confirm_overwrite(".gitignore") {
        println!("Aborted.");
        return Ok(());
    }

    match app::run_wizard(&project_name)? {
        Some(answers) => {
            generator::generate(&project_path, &answers)?;
            println!("Created: {}", project_path.display());
        }
        None => {
            println!("Cancelled.");
        }
    }

    Ok(())
}