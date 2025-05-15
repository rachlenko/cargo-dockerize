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
main.rs

## Advanced Use Cases

Here's an example showing how to use the cargo dockerize command with all the available command-line arguments:

``` bash
cargo dockerize \
  --export \
  --name my-rust-app \
  --tag 1.2.3 \
  --dockerfile Dockerfile.prod \
  --tags latest,stable,1.2 \
  --application_name "My Rust Application" \
  --title "My Awesome Rust Application" \
  --description "A Rust application that does amazing things" \
  --authors "Jane Doe <jane@example.com>, John Smith <john@example.com>" \
  --url "https://my-app.example.com" \
  --source "https://github.com/username/my-rust-app" \
  --vendor "Example Corporation" \
  --licenses "MIT"
  ```

This command will:

Build the Rust project with cargo build --release
Build a Docker image with:

Name: my-rust-app
Primary tag: 1.2.3
Additional tags: latest, stable, and 1.2
Using Dockerfile.prod instead of the default Dockerfile


Apply the following OCI labels to the Docker image:

org.opencontainers.image.title=My Awesome Rust Application
org.opencontainers.image.description=A Rust application that does amazing things
org.opencontainers.image.version=1.2.3
org.opencontainers.image.created=[current UTC timestamp]
org.opencontainers.image.authors=Jane Doe <jane@example.com>, John Smith <john@example.com>
org.opencontainers.image.url=https://my-app.example.com
org.opencontainers.image.source=https://github.com/username/my-rust-app
org.opencontainers.image.revision=[git commit hash]
org.opencontainers.image.vendor=Example Corporation
org.opencontainers.image.licenses=MIT
application_name=My Rust Application


Export the Docker image as a TGZ file named my-rust-app-1.2.3.tgz

You can also use a shorter version if you're fine with some defaults:

```
cargo dockerize --name my-app --tag 0.1.0 --description "My app description" --authors "Your Name" --source "https://github.com/you/my-app"
```
This would use defaults for other parameters, like using the standard Dockerfile in the project root, not exporting to TGZ, and using the package name and version from Cargo.toml as fallbacks if --name or --tag weren't specified.


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
