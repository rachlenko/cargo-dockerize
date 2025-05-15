use std::path::{Path, PathBuf};
use std::process::{Command, exit};
use std::env;
use std::fs;
use anyhow::{Result, Context, bail};
use clap::{Parser, Subcommand};
use chrono::Utc;

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
    
    /// Additional tags for the Docker image
    #[arg(long, value_delimiter = ',')]
    tags: Vec<String>,
    
    /// Application name
    #[arg(long)]
    application_name: Option<String>,
    
    /// OCI Image title
    #[arg(long)]
    title: Option<String>,
    
    /// OCI Image description
    #[arg(long)]
    description: Option<String>,
    
    /// OCI Image authors
    #[arg(long)]
    authors: Option<String>,
    
    /// OCI Image URL
    #[arg(long)]
    url: Option<String>,
    
    /// OCI Image source repository
    #[arg(long)]
    source: Option<String>,
    
    /// OCI Image vendor
    #[arg(long)]
    vendor: Option<String>,
    
    /// OCI Image licenses
    #[arg(long)]
    licenses: Option<String>,
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
    
    // Get git revision if available
    let git_revision = get_git_revision(&project_root).unwrap_or_else(|_| String::from("unknown"));
    
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
    
    // Prepare Docker build command with OCI labels
    let mut docker_build_args = vec![
        "build".to_string(),
        "-t".to_string(),
        image_full.clone(),
        "-f".to_string(),
        args.dockerfile.clone(),
    ];
    
    // Add additional tags if specified
    for tag in &args.tags {
        docker_build_args.push("-t".to_string());
        docker_build_args.push(format!("{}:{}", image_name, tag));
    }
    
    // Add OCI labels
    let current_time = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    
    // Add standard OCI labels
    add_label(&mut docker_build_args, "org.opencontainers.image.created", &current_time);
    add_label(&mut docker_build_args, "org.opencontainers.image.version", &image_tag);
    add_label(&mut docker_build_args, "org.opencontainers.image.revision", &git_revision);
    
    // Add optional OCI labels if provided
    if let Some(title) = &args.title {
        add_label(&mut docker_build_args, "org.opencontainers.image.title", title);
    } else {
        add_label(&mut docker_build_args, "org.opencontainers.image.title", &image_name);
    }
    
    if let Some(desc) = &args.description {
        add_label(&mut docker_build_args, "org.opencontainers.image.description", desc);
    }
    
    if let Some(authors) = &args.authors {
        add_label(&mut docker_build_args, "org.opencontainers.image.authors", authors);
    }
    
    if let Some(url) = &args.url {
        add_label(&mut docker_build_args, "org.opencontainers.image.url", url);
    }
    
    if let Some(source) = &args.source {
        add_label(&mut docker_build_args, "org.opencontainers.image.source", source);
    }
    
    if let Some(vendor) = &args.vendor {
        add_label(&mut docker_build_args, "org.opencontainers.image.vendor", vendor);
    }
    
    if let Some(licenses) = &args.licenses {
        add_label(&mut docker_build_args, "org.opencontainers.image.licenses", licenses);
    }
    
    // Add application_name label if provided
    if let Some(app_name) = &args.application_name {
        add_label(&mut docker_build_args, "application_name", app_name);
    }
    
    // Add the build context
    docker_build_args.push(".".to_string());
    
    // Build Docker image
    println!("Building Docker image: {}...", image_full);
    let docker_build_status = Command::new("docker")
        .current_dir(&project_root)
        .args(&docker_build_args)
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

// Helper function to add a label to docker build args
fn add_label(args: &mut Vec<String>, key: &str, value: &str) {
    args.push("--label".to_string());
    args.push(format!("{}={}", key, value));
}

// Get git revision (commit hash)
fn get_git_revision(project_root: &Path) -> Result<String> {
    let output = Command::new("git")
        .current_dir(project_root)
        .args(["rev-parse", "HEAD"])
        .output()
        .context("Failed to execute git command")?;
    
    if output.status.success() {
        let hash = String::from_utf8(output.stdout)
            .context("Invalid UTF-8 in git output")?
            .trim()
            .to_string();
        Ok(hash)
    } else {
        bail!("Git command failed")
    }
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
