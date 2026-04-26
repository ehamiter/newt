use clap::Parser;
use std::path::PathBuf;

mod app;
mod generator;
mod ui;

#[derive(Parser)]
#[command(name = "newt", version, about = "Scaffold a new project with .devcontainer setup")]
struct Cli {
    /// Project name — a directory with this name will be created here
    project_name: String,

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
    // Check for valid Unix filename characters
    let invalid_chars = ['\0', '/', '\\'];
    for c in invalid_chars {
        if name.contains(c) {
            return Err(format!("Project name contains invalid character: {}", c));
        }
    }
    // Check for whitespace
    if name.chars().any(|c| c.is_whitespace()) {
        return Err("Project name cannot contain whitespace".into());
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Validate project name
    if let Err(e) = validate_project_name(&cli.project_name) {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }

    let cwd = match cli.output {
        Some(p) => p,
        None => std::env::current_dir()?,
    };
    let project_path = cwd.join(&cli.project_name);

    match app::run_wizard(&cli.project_name)? {
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