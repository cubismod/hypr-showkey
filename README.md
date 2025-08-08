# Hypr-showkey

A fast, configurable TUI application for displaying and searching Hyprland keybindings with fuzzy search functionality.

## Features

- 🔍 **Fuzzy Search** - Quickly find keybindings by typing partial matches
- 📊 **Responsive Columns** - Automatically adapts to terminal width (50+ chars per column)
- 🎨 **Catppuccin Themes** - Beautiful color themes with full customization
- 📁 **Configurable** - Parse any Hyprland config files you specify
- 🏷️ **Categorization** - Automatically categorize keybindings by function
- 🚫 **Smart Filtering** - Unbound keybindings are automatically filtered out
- ⚡ **Fast** - Built in Rust for speed and reliability
- 🎯 **Easy Navigation** - Vim-like keybindings for smooth navigation

## Installation

### Quick Setup with Task Runner

1. Clone the repository:
```bash
git clone <repository-url>
cd hypr-showkey
```

2. Set up the development environment:
```bash
task setup
```

3. Install the application:
```bash
task install          # Install to ~/.local/bin
# OR
task install-global   # Install globally with cargo
```

### Manual Installation

1. Clone and build:
```bash
git clone <repository-url>
cd hypr-showkey
cargo build --release
```

2. Install manually:
```bash
cp target/release/hypr-showkey ~/.local/bin/
```

## Configuration

Create a configuration file at `~/.config/showkey.yaml`:

```yaml
# Hyprland configuration files to parse
hyprland_configs:
  files:
    - "conf/keybindings/default.conf"
    - "conf/keybindings/custom.conf"
    # Add your Hyprland config files here

# Optional: Categorize keybindings
categories:
  applications:
    name: "Applications"
    description: "Launch applications and tools"
    keywords: ["terminal", "browser", "filemanager"]

# UI settings
ui:
  show_descriptions: true
  search_threshold: 0.6
  max_results: 50
  
  # Theme (Catppuccin Mocha by default)
  theme:
    name: "catppuccin_mocha"
    colors:
      key_color: "#89b4fa"        # Blue
      action_color: "#cdd6f4"     # Text
      category_color: "#a6e3a1"   # Green
      # ... more colors
```

## Usage

```bash
# Use default config location (~/.config/showkey.yaml)
hypr-showkey

# Use custom config file
hypr-showkey --config /path/to/config.yaml

# With Task runner
task dev                    # Run in development mode
task dev-config -- custom.yaml  # Run with custom config
```

## Development

This project uses [Task](https://taskfile.dev/) for automation. Common commands:

```bash
task                    # Show all available tasks
task dev               # Run in development mode
task build-release     # Build optimized binary
task check             # Check code without building
task test              # Run tests
task lint              # Run clippy linter
task fmt               # Format code
task qa                # Run all quality checks
task install           # Install to ~/.local/bin
task setup-config      # Copy example config
```

### Keybindings

- **Navigation**: `↑/k` (up), `↓/j` (down)
- **Search**: Type to search keybindings
- **Help**: `?` or `F1` to toggle help
- **Quit**: `q` or `Esc`
- **Clear Search**: `Backspace`

### Display Features

- **Responsive Layout**: Automatically creates multiple columns based on terminal width
- **Smart Filtering**: Unbound keybindings (using `unbind` or empty actions) are hidden
- **Minimum Width**: Each column requires at least 50 characters for readability

## Themes

Hypr-showkey comes with built-in Catppuccin themes:

- `catppuccin_mocha` (default, dark)
- `catppuccin_latte` (light)
- `catppuccin_macchiato` (dark)
- `catppuccin_frappe` (dark)

You can also define custom colors using hex values in your config file.

## Configuration File Paths

The app supports both relative paths (relative to `~/.config/hypr/`) and absolute paths for Hyprland configuration files:

```yaml
hyprland_configs:
  files:
    - "conf/keybindings/default.conf"      # Relative to ~/.config/hypr/
    - "/absolute/path/to/config.conf"      # Absolute path
```

## License

Licensed under the Apache License, Version 2.0
