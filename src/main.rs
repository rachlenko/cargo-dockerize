use std::path::{Path, PathBuf};
use std::process::{Command, exit};
use std::env;
use std::fs;
use anyhow::{Result, Context, bail};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
enum Cargo {
    #[command(name = "dockerize")]
    Dockerize(Dockerize),
}

#[derive(clap::Args)]
#[command(author, version, about, long_about = None)]
struct Dockerize {
    /// Export the Docker image as a TGZ archive
    #[arg(short, long)]
    export: bool,
    
    /// Name of the Docker image (defaults to the package name)
    #[arg(short, long)]
    name: Option<String>,
    
    /// Version tag of the Docker image (defaults to the package version)
    #[arg(short, long)]
    tag: Option<String>,
    
    /// Path to the Dockerfile (defaults to ./Dockerfile)
    #[arg(long, default_value = "Dockerfile")]
    dockerfile: String,
}

fn main() -> Result<()> {
    let Cargo::Dockerize(args) = Cargo::parse();
    
    // Find cargo project root
    let project_root = find_project_root()?;
    println!("Project root: {}", project_root.display());
    
    // Read cargo metadata to get package info
    let metadata = get_cargo_metadata(&project_root)?;
    
    // Determine image name and tag
    let image_name = args.name.unwrap_or_else(|| metadata.0.clone());
    let image_tag = args.tag.unwrap_or_else(|| metadata.1.clone());
    let image_full = format!("{}:{}", image_name, image_tag);
    
    // Verify Dockerfile exists
    let dockerfile_path = project_root.join(&args.dockerfile);
    if !dockerfile_path.exists() {
        bail!("Dockerfile not found at: {}", dockerfile_path.display());
    }
    
    // Build the Rust project
    println!("Building Rust project...");
    let build_status = Command::new("cargo")
        .current_dir(&project_root)
        .args(["build", "--release"])
        .status()
        .context("Failed to execute cargo build")?;
        
    if !build_status.success() {
        bail!("Cargo build failed");
    }
    
    // Build Docker image
    println!("Building Docker image: {}...", image_full);
    let docker_build_status = Command::new("docker")
        .current_dir(&project_root)
        .args(["build", "-t", &image_full, "-f", &args.dockerfile, "."])
        .status()
        .context("Failed to execute docker build")?;
        
    if !docker_build_status.success() {
        bail!("Docker build failed");
    }
    
    // Export to TGZ if requested
    if args.export {
        let archive_name = format!("{}-{}.tgz", image_name, image_tag);
        let archive_path = project_root.join(&archive_name);
        println!("Exporting Docker image to: {}...", archive_path.display());
        
        let export_status = Command::new("sh")
            .current_dir(&project_root)
            .arg("-c")
            .arg(format!("docker save {} | gzip > {}", image_full, archive_name))
            .status()
            .context("Failed to export Docker image")?;
            
        if !export_status.success() {
            bail!("Docker export failed");
        }
        
        println!("Docker image exported successfully to: {}", archive_path.display());
    }
    
    println!("Dockerize completed successfully!");
    Ok(())
}

// Find the root of the cargo project
fn find_project_root() -> Result<PathBuf> {
    let mut current_dir = env::current_dir().context("Failed to get current directory")?;
    
    loop {
        let cargo_toml = current_dir.join("Cargo.toml");
        if cargo_toml.exists() {
            return Ok(current_dir);
        }
        
        if !current_dir.pop() {
            bail!("Could not find Cargo.toml in any parent directory");
        }
    }
}

// Get package name and version from Cargo.toml
fn get_cargo_metadata(project_root: &Path) -> Result<(String, String)> {
    let cargo_toml_path = project_root.join("Cargo.toml");
    let cargo_toml_content = fs::read_to_string(cargo_toml_path)
        .context("Failed to read Cargo.toml")?;
    
    let name_line = cargo_toml_content.lines()
        .find(|line| line.trim().starts_with("name ="))
        .context("Could not find package name in Cargo.toml")?;
    
    let version_line = cargo_toml_content.lines()
        .find(|line| line.trim().starts_with("version ="))
        .context("Could not find package version in Cargo.toml")?;
    
    let name = name_line.split('=').nth(1)
        .context("Invalid name format in Cargo.toml")?
        .trim()
        .trim_matches('"')
        .to_string();
    
    let version = version_line.split('=').nth(1)
        .context("Invalid version format in Cargo.toml")?
        .trim()
        .trim_matches('"')
        .to_string();
    
    Ok((name, version))
}
