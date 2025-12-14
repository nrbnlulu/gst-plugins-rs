# gst-plugins-rs QWEN.md

## Project Overview

**gst-plugins-rs** is a repository containing various GStreamer plugins and elements written in the Rust programming language. The project builds upon the GStreamer Rust bindings and provides implementations of multimedia processing plugins in Rust.

### Key Features
- Multiple plugin categories: audio, video, network, generic, text, utilities, and analytics
- Extensive set of plugins for different multimedia use cases:
  - Video codecs: dav1d (AV1 decoder), rav1e (AV1 encoder), vvdec (VVC/H.266 decoder), ffv1
  - Audio processing: audio effects, parsers, noise reduction, speech-to-text
  - Network protocols: RTSP, WebRTC, AWS services, NDI, QUIC
  - Multimedia formats: GIF, PNG, WebP encoders/decoders
  - Utility elements: file operations, stream grouping, fallback switches

### Architecture
- Organized as a Rust workspace with multiple plugin crates
- Uses Meson build system for integration with GStreamer ecosystem
- Depends on cargo-c for creating C-compatible shared libraries
- Supports both static and dynamic linking options

## Building and Running

### Prerequisites
```bash
# Install cargo-c for building C libraries
cargo install cargo-c
```

### Build Methods

#### Method 1: Using Cargo directly
```bash
# Build all plugins
cargo build

# Build release version
cargo build --release

# Build a specific plugin
cargo cbuild -p gst-plugin-cdg

# Install system-wide
cargo cinstall -p gst-plugin-cdg --prefix=/usr
```

#### Method 2: Using Meson
```bash
# Initialize build directory
meson setup builddir/

# Build all enabled plugins
ninja -C builddir/

# Install
ninja -C builddir/ install
```

#### Method 3: Using Makefile
```bash
# Build release
make

# Build debug
make DEBUG=1

# Install
make install
```

### Testing
```bash
# Run tests for a specific plugin
cargo test -p gst-plugin-cdg

# Run with Meson
meson test -C builddir/
```

### Configuration Options
The Meson build system supports various options:
- Plugin-specific options to enable/disable individual plugins
- Feature flags for optional functionality
- Support for external dependencies like gtk4, csound, etc.

## Development Conventions

### Code Organization
- Each plugin is in its own subdirectory under category directories (audio/, video/, net/, etc.)
- Plugin code follows Rust naming conventions and idioms
- Integration with GStreamer via official Rust bindings

### Licensing
- MPL-2.0 (preferred for new plugins)
- Apache License 2.0
- MIT License

### Dependencies
- Uses GStreamer Rust bindings from git repository
- External dependencies managed via Cargo.toml files per plugin
- Workspace dependencies defined in the root Cargo.toml

### Release Strategy
- Releases from versioned branches (e.g., `0.14`)
- Git tags for each release (e.g., `0.14.2`)
- Published to crates.io for distribution
- Internal `gstreamer-X.Y.Z` tags for binary releases

## Important Files and Directories

- `Cargo.toml` - Root workspace configuration with member plugins
- `meson.build` - Main Meson build configuration
- `Makefile` - Simple Make-based build interface
- `README.md` - Main project documentation
- Category directories (audio/, video/, net/, etc.) - Plugin organization
- `cargo_wrapper.py` - Script for integrating cargo with Meson
- `dependencies.py` - Script for managing plugin dependencies

## Plugin Categories

### Generic Plugins
- file: Rust implementation of filesrc and filesink
- inter: Wormhole for data transfer within process
- sodium: Encryption/decryption elements
- threadshare: Threaded elements with common infrastructure

### Audio Plugins  
- audiofx: Effects like echo, normalization, noise removal
- claxon: FLAC decoder
- lewton: Vorbis decoder
- csound: Csound integration

### Video Plugins
- dav1d: AV1 decoder
- rav1e: AV1 encoder
- gif: GIF encoder
- png: PNG encoder
- hsv: Color space transformation filters

### Network Plugins
- aws: Amazon AWS services integration (S3, Transcribe)
- webrtc: WebRTC elements
- rtsp: RTSP source implementation
- ndi: NewTek NDI integration

### Utility Plugins
- fallbackswitch: Fallback mechanisms for unstable sources
- livesync: Live stream maintenance
- tracers: Various tracing capabilities for debugging