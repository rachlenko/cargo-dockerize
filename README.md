# Cargo Dockerize

A Cargo subcommand to build Rust applications as Docker images and export them as TGZ archives.

## Features

- Build your Rust application with Cargo
- Create a Docker image with your application
- Optionally export the Docker image as a TGZ archive
- Customize image name, tag, and Dockerfile path
- Automatic detection of project information from Cargo.toml

## Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/cargo-dockerize.git
cd cargo-dockerize

# Install the cargo subcommand
cargo install --path .
```

## Usage

Once installed, you can use the `cargo dockerize` command in any Rust project:

```bash
# Build Docker image with default settings
cargo dockerize

# Build Docker image and export as TGZ
cargo dockerize --export

# Specify custom image name and tag
cargo dockerize --name my-app --tag 1.0.0

# Use a custom Dockerfile
cargo dockerize --dockerfile Dockerfile.prod
```

## Command-line Options

| Option | Description |
|--------|-------------|
| `-e, --export` | Export the Docker image as a TGZ archive |
| `-n, --name NAME` | Custom name for the Docker image (defaults to package name) |
| `-t, --tag TAG` | Custom tag for the Docker image (defaults to package version) |
| `--dockerfile PATH` | Path to the Dockerfile (defaults to ./Dockerfile) |
| `-h, --help` | Print help information |
| `-V, --version` | Print version information |

## Project Setup

### Directory Structure

Your project should have the following structure:

```
your-rust-project/
├── Cargo.toml          # Standard Cargo.toml with name and version
├── Dockerfile          # Dockerfile for building your application
└── src/
    └── main.rs         # Your Rust application
```

### Example Dockerfile

Here's a sample Dockerfile that works well with cargo-dockerize:

```dockerfile
# Build stage
FROM rust:1.76-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev

# Create a new empty project
WORKDIR /usr/src/app
COPY . .

# Build for release
RUN cargo build --release

# Runtime stage
FROM alpine:3.19

# Install runtime dependencies (if any)
RUN apk add --no-cache ca-certificates

# Copy the binary from the build stage
COPY --from=builder /usr/src/app/target/release/your_app_name /usr/local/bin/

# Run the binary
CMD ["your_app_name"]
```

Remember to replace `your_app_name` with your application name from Cargo.toml.

## Implementation Details

The cargo-dockerize command performs the following steps:

1. Finds the root of your Cargo project
2. Reads metadata from your Cargo.toml file
3. Builds your Rust project with `cargo build --release`
4. Builds a Docker image using your Dockerfile
5. Optionally exports the Docker image as a TGZ archive

## Code Implementation

The core implementation of cargo-dockerize is quite straightforward:

```rust
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
```

## Advanced Use Cases

### Configuration File

You can create a `.dockerize.toml` file in your project root to customize the defaults:

```toml
[dockerize]
name = "custom-name"  # Override the default image name
tag = "latest"        # Override the default tag
dockerfile = "Dockerfile.prod"  # Use a different Dockerfile
```

### CI/CD Integration

Cargo Dockerize works well in CI/CD pipelines. Example GitHub Actions workflow:

```yaml
name: Build and Dockerize

on:
  push:
    branches: [ main ]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install cargo-dockerize
        run: cargo install --git https://github.com/yourusername/cargo-dockerize.git
      - name: Build and export Docker image
        run: cargo dockerize --export
      - name: Upload artifact
        uses: actions/upload-artifact@v3
        with:
          name: docker-image
          path: "*.tgz"
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
