# 🚀 PkgMgr: The Definitive CLI Command Catalog for Linux

[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Arch Linux Compatibility](https://img.shields.io/badge/Compatible-Arch%20Linux-brightgreen.svg)]()

> Overwhelmed by the sheer number of CLI commands installed on your system? `PkgMgr` is a personal catalog manager designed to **inventory, organize, and recall** all your favorite Linux command-line utilities, making the terminal experience more accessible and intuitive.

---

## ✨ What Problem Does PkgMgr Solve?

In a modern Linux environment like Arch, it's easy to install dozens of CLI tools (from `git` to specialized microservices). Over time, remembering the exact command or understanding what an executable does can become a tedious task.

`PkgMgr` solves this by creating a **centralized and navigable database** of all your favorite commands. Instead of searching Google or trying to recall complex names, you simply use `pkg` to query, list, or execute what you need.

## 💡 Key Features

*   **Central Inventory:** Keep a single record of every CLI program you use.
*   **Intelligent Search:** Search by command name, description, or even custom categories.
*   **Personal Categorization:** Assign detailed descriptions and classify tools into relevant categories (e.g., `Development`, `Networking`, `Utilities`).
*   **Direct Execution:** Not just a catalog; it also allows direct execution of registered AppImage files or binaries.
*   **Scalable:** Designed to be the control center for your CLI workflow, with future support for more formats (beyond Arch/Pacman).

## 🛠️ Installation and Setup

The installation is currently achieved by downloading the executable binary and manually adding its location to your `$PATH`, as it is not yet packaged for distribution via tools like pacman or cargo.

**1. Install the Package:**
*(Assuming you are in an environment where `pkgmgr` can be installed)*

```bash
# Download and install the binary manually, then add it to your PATH.
# Example: tar -xzf pkgmgr-vX.Y.tar.gz && sudo mv pkgmgr /usr/local/bin
```

**2. Verify Installation:**
Once placed in `$PATH`, the main command will be available:

```bash
pkg --help
```

> **Important Note:** The executable is named `pkgmgr`, but to simplify your workflow, you can use it as the alias **`pkg`**.

## 🚀 Getting Started

Use these commands to start cataloging your tools!

### 1. List Known Packages
To see what you have registered so far:

```bash
pkg list
# or its alias: pkg ls
```

### 2. Add a Command (Recommended Method: `install`)
Advanced example: Registering a package and customizing its name or description with official system information.

```bash
# General syntax: pkg install <package> --command <DESIRED_CMD> --description "<MY_DETAILS>" [--category <CAT>]
pkg install htop --command topmon --description "Interactive system resource monitor, better than top." --category System
```

**Note:** The `install` command is the preferred method because it extracts official package metadata (default category 'System', description, etc.), but allows overriding details with additional parameters.

### 3. Search for a Specific Command
If you need to know what your networking tool does:

```bash
pkg search networking
# or pkg -q networking
```

## 📚 Available Commands (Reference)

`PkgMgr` is designed with the following subcommands to handle all your needs:

| Command | Alias(s) | Description | Recommended Use Case |
| :--- | :--- | :--- | :--- |
| **`install`** | `i` | Installs with pacman and registers in the catalog. This is the preferred method, as it extracts official package metadata. It can be customized with flags like `--command <CMD>` or `--description "<DESCRIPTION>"`. | Use whenever you know the package name (e.g., \`pkg install htop\`). Ideal for capturing full metadata and customizing details. |
| **`list`** | `ls` | Lists all packages registered in the catalog. | General inventory review. |

## 🔮 Roadmap

*   ✅ Advanced support for AppImages and external binaries.
*   ✨ Interactive selector mode: A guided menu that allows you to choose tools without remembering commands.
*   🧩 Enhanced category and tag management.

---
_This project is a personal catalog designed by the user to improve the command-line experience._

**Contributions:** Be part of the effort! Help us catalog more commands and features.