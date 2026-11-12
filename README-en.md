# 🚀 PkgMgr: a personal catalog of CLI commands for Linux

[![Arch Linux](https://img.shields.io/badge/Arch%20Linux-compatible-brightgreen.svg)]()
[![Rust](https://img.shields.io/badge/Rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)

**Languages:** English · [Español](README.md)

> Struggling to remember all the CLI commands installed on your system? `PkgMgr` is a personal catalog designed to **inventory, organize, and recall** your command-line utilities, making the terminal easier to use.

---

## ✨ What problem does it solve?

In a modern Linux environment like Arch Linux, it is easy to accumulate dozens of CLI tools, from `git` to highly specialized utilities. Over time, remembering the exact command or knowing what an executable does can become tedious.

`PkgMgr` solves this with a **local, centralized, searchable database** of your favorite commands. Instead of searching the web or straining your memory, you use `pkg` to query, list, or run what you need.

## 💡 Features

- **Central inventory:** a single record of every CLI program you use.
- **Fast search:** by name, description, category, command, or origin (case-insensitive).
- **Free-form categories:** classify tools however you like, with suggested categories available.
- **pacman integration:** pulls the official package description and version.
- **AppImages:** installs them into `~/Apps` with `700` permissions and runs them by catalog name.
- **No extra dependencies:** only `pacman` is required (plus `sudo` to install packages).

## 📋 Requirements

- **Arch Linux** or a distribution with `pacman`.
- **Rust 1.85 or newer** to build (the project uses the 2024 edition).
- **`sudo`** to install packages with `pkg install`.

## 🛠️ Installation

The binary is called **`pkg`**. Build it from the project directory:

```bash
cargo build --release
sudo install -Dm755 target/release/pkg /usr/local/bin/pkg
```

You can also install it for your user only with Cargo:

```bash
cargo install --path .
```

Verify it works:

```bash
pkg --version
pkg --help
```

## 🚀 Getting started

```bash
# Show the catalog (running `pkg` with no subcommand does the same)
pkg list

# Install a package with pacman and register it
pkg install htop -c System

# Search for a tool by name, description, or category
pkg search network
```

## 📚 Commands

| Command   | Alias           | Description                                                                 |
| :-------- | :-------------- | :-------------------------------------------------------------------------- |
| `add`     | —               | Registers an **already installed** package in the catalog (installs nothing). |
| `install` | `i`             | Installs with pacman and registers it; given a `.AppImage`, installs it into `~/Apps` and registers it. |
| `list`    | `ls`            | Lists the packages in the catalog.                                          |
| `search`  | `-q`, `--query` | Searches by name, description, category, command, or origin.                |
| `info`    | —               | Shows a package's details and refreshes its metadata from pacman.           |
| `remove`  | —               | Removes a package from the catalog (does not uninstall it from the system). |
| `execute` | `e`             | Runs an AppImage registered in the catalog.                                 |

### Syntax

```text
pkg add     [OPTIONS] [PACKAGE]...
pkg install [OPTIONS] [PACKAGE]...
pkg list
pkg search  <TEXT>           # also: pkg -q <TEXT> | pkg --query <TEXT>
pkg info    [PACKAGE]...
pkg remove  [PACKAGE]...
pkg execute <PACKAGE>        # also: pkg e <PACKAGE>
```

> **Note:** running `pkg` with no subcommand prints the catalog (same as `pkg list`).

### Common options for `add` and `install`

| Option                     | Description                                                                                                                  |
| :------------------------- | :--------------------------------------------------------------------------------------------------------------------------- |
| `-c, --category [CAT]`     | Personal category. With no value (`pkg add -c`) it lists categories in use and suggested ones.                               |
| `--command <CMD>`          | Command used to invoke the tool when it differs from the package name (for example, `zoxide` → `z`). For AppImages it sets the catalog name. |
| `-d, --description <TEXT>` | Personal description; takes precedence over pacman's.                                                                        |
| `-o, --origin <ORIGIN>`    | Record origin (`manual`, `pacman`, `appimage`, or free text). Defaults: `manual` in `add`, `pacman` in `install`, and `appimage` for AppImages. |

`install` accepts multiple packages at once. If a package is not in the repositories, it warns and **only registers it** (useful for AUR or other sources). For AppImages, if `--command` or `-d` is missing, you are prompted interactively.

## 🧪 Examples

```bash
# Register an already installed tool
pkg add fzf -c Terminal -d "Fuzzy finder for the terminal"

# Install and register with pacman metadata
pkg install htop -c System

# Register a command that differs from the package name
pkg install zoxide --command z -c Terminal

# Register an AUR or otherwise installed package (not installed, only registered)
pkg install paru -o AUR

# Show categories in use and suggested ones
pkg add -c
```

### AppImages

```bash
# Install the AppImage into ~/Apps (700 permissions) and register it as "gimp"
pkg install ~/Downloads/GIMP-2.10.34-x86_64.AppImage \
  --command gimp -c Other -d "Image editor"

# Run it from the catalog
pkg execute gimp
pkg e gimp
```

> The `.AppImage` file is **moved** into `~/Apps`; no copy is kept at the original location. If the destination already exists, you are asked before overwriting it. Only registered AppImages can be run, and `pkg execute` does not accept additional arguments for the app.

### Inspect and remove

```bash
pkg info gimp     # description, version, category, command, and origin
pkg remove gimp   # removes it from the catalog only; nothing is uninstalled
```

## 🗄️ Data and storage

- **Database:** `$XDG_DATA_HOME/pkg/pkg.db` (by default, `~/.local/share/pkg/pkg.db`). It is SQLite and is created automatically.
- **AppImages:** `~/Apps`.
- **Suggested categories:** `Development`, `Files`, `Git`, `System`, `Terminal`, `Network`, `Other`. Any other value is allowed.
- **`list` and `search` output:** name, category, command in brackets, and description.

## 🔮 Roadmap

- ✨ Interactive selector: a guided menu to pick tools without remembering commands.
- 🧩 Tags in addition to categories.
- 🐧 Support for package managers other than pacman.

## 🤝 Contributing

Contributions are welcome! Feel free to open an issue or a pull request with improvements, fixes, or new ideas for the catalog.

---

_A personal project created to improve the command-line experience._
