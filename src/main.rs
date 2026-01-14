use indicatif::{ProgressBar, ProgressStyle};
mod iostream;
mod parse;
use clap::Parser;
use clap::Subcommand;
use std::path::PathBuf;
use std::{fs::File, io::Write, io::Read};
use reqwest::blocking::Client;


#[derive(Parser)]
#[command(name = "tmpl")]
#[command(about = "A template processing tool", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    tmpl: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    Install {
        name: String,
    },
    Remove {
        name: String,
    },
    List {}
}

fn main() {
    let args = Args::parse();

    match &args.command {
        Some(Commands::Install { name }) => {
            if let Err(e) = download_tmpl(name) {
                eprintln!("Error downloading template: {}", e);
            }
        }
        Some(Commands::Remove { name }) => {
            remove_tmpl(name);
        }
        Some(Commands::List {}) => {
            list_tmpls();
        }
        None => {
            if let Some(tmpl_name) = &args.tmpl {
                if let Err(e) = use_tmpl(tmpl_name) {
                    eprintln!("Error using template '{}': {}", tmpl_name, e);
                } 
            } else {
                eprintln!("No template name provided. Use `tmpl <name>` or `tmpl install <name>`.");
            }
        }
    }  
}

fn list_tmpls() {
    let templates_dir: PathBuf = dirs::data_dir()
        .expect("Could not find data directory")
        .join("tmpl/templates");

    if templates_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(templates_dir) {
            println!("Installed templates:");
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    println!(" - {}", name);
                }
            }
        } else {
            eprintln!("No templates installed.");
        }
    }
}

fn download_tmpl(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let name = name.trim();

    if name.is_empty() {
        return Err("Template name cannot be empty".into());
    }

    if name.contains(' ') {
        return Err("Template name cannot contain spaces".into());
    }

    if name == "." {
        copy_current_dir_template()?;
        return Ok(());
    }

    let url = format!(
        "https://raw.githubusercontent.com/Jamie-Poeffel/tmpl/refs/heads/registry/{}/file.tmpl",
        name
    );

    let client = Client::new();
    let mut response = client.get(&url).send()?;

    if !response.status().is_success() {
        return Err(format!("Failed to download template '{}'", name).into());
    }

    let total_size = response.content_length().unwrap_or(0);
    let pb = ProgressBar::new(total_size);

    pb.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})",
        )?
        .progress_chars("#>-"),
    );

    let mut dest_path: PathBuf = dirs::data_dir()
        .expect("Could not find data directory")
        .join("tmpl/templates")
        .join(name);

    std::fs::create_dir_all(&dest_path)?;

    dest_path.push("file.tmpl");

    let mut file = File::create(&dest_path)?;
    let mut downloaded = 0u64;
    let mut buffer = [0u8; 8192];

    while let Ok(n) = response.read(&mut buffer) {
        if n == 0 {
            break;
        }
        file.write_all(&buffer[..n])?;
        downloaded += n as u64;
        pb.set_position(downloaded);
    }

    pb.finish_with_message(format!("\x1b[32m√\x1b[0m Template '{}' downloaded to {}", name, dest_path.display()));
    Ok(())
}

fn remove_tmpl(name: &str) {
    let mut tmpl_path: PathBuf = dirs::data_dir()
        .expect("Could not find data directory")
        .join("tmpl/templates")
        .join(name);

    if tmpl_path.exists() {
        if let Err(e) = std::fs::remove_dir_all(&tmpl_path) {
            eprintln!("Error removing template '{}': {}", name, e);
        } else {
            println!("\x1b[32m√\x1b[0m Template '{}' removed", name);
        }
    } else {
        eprintln!("Template '{}' does not exist", name);
    }
}

fn use_tmpl(template: &str) -> Result<(), Box<dyn std::error::Error>> {
    let _ = parse::parse_template(template)?;
    Ok(())
}

fn copy_current_dir_template() -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = std::env::current_dir()?;
    
    let tmpl_files: Vec<_> = std::fs::read_dir(&current_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == "tmpl")
                .unwrap_or(false)
        })
        .collect();

    if tmpl_files.is_empty() {
        println!("No .tmpl files found in current directory");
        return Ok(());
    }

    let selected_file = if tmpl_files.len() > 1 {
        println!("Available templates:");
        for (i, file) in tmpl_files.iter().enumerate() {
            println!("  {}. {}", i + 1, file.file_name().to_string_lossy());
        }

        let selection = iostream::get_input_text(
            "Select template number",
            "1"
        )?;

        let index: usize = selection.trim().parse::<usize>()
            .map_err(|_| "Invalid selection")?
            .saturating_sub(1);

        if index >= tmpl_files.len() {
            return Err("Invalid template selection".into());
        }

        &tmpl_files[index]
    } else {
        &tmpl_files[0]
    };
    
    let template_name = iostream::get_input_text(
        "Enter name for this template",
        selected_file.path()
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("template")
    )?;

    let mut dest_path: PathBuf = dirs::data_dir()
        .expect("Could not find data directory")
        .join("tmpl/templates")
        .join(&template_name);

    std::fs::create_dir_all(&dest_path)?;

    dest_path.push("file.tmpl");
    std::fs::copy(selected_file.path(), &dest_path)?;

    println!("\x1b[32m√\x1b[0m Template copied as '{}'", template_name);

    Ok(())
}