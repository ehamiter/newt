use clap::Parser;

mod app;
mod generator;
mod ui;

#[derive(Parser)]
#[command(name = "newt", about = "Scaffold a new project with .devcontainer setup")]
struct Cli {
    /// Project name — a directory with this name will be created here
    project_name: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let cwd = std::env::current_dir()?;
    let project_path = cwd.join(&cli.project_name);

    if project_path.exists() {
        eprintln!("error: '{}' already exists", project_path.display());
        std::process::exit(1);
    }

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