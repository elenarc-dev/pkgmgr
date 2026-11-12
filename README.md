# 🚀 PkgMgr: catálogo personal de comandos CLI para Linux

[![Arch Linux](https://img.shields.io/badge/Arch%20Linux-compatible-brightgreen.svg)]()
[![Rust](https://img.shields.io/badge/Rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)

**Idiomas:** Español · [English](README-en.md)

> ¿Te cuesta recordar todos los comandos CLI que tienes instalados? `PkgMgr` es un catálogo personal diseñado para **inventariar, organizar y recordar** tus utilidades de línea de comandos, haciendo que la terminal sea más fácil de usar.

---

## ✨ ¿Qué problema resuelve?

En un entorno Linux moderno como Arch Linux, es fácil acumular decenas de herramientas CLI, desde `git` hasta utilidades muy especializadas. Con el tiempo, recordar el comando exacto o saber qué hace un ejecutable puede volverse tedioso.

`PkgMgr` resuelve esto con una **base de datos local, centralizada y consultable** de tus comandos favoritos. En lugar de buscar en internet o exprimir la memoria, usas `pkg` para consultar, listar o ejecutar lo que necesitas.

## 💡 Características

- **Inventario centralizado:** un único registro de todos los programas CLI que usas.
- **Búsqueda rápida:** por nombre, descripción, categoría, comando u origen, sin distinguir mayúsculas.
- **Categorías libres:** clasifica las herramientas como prefieras; hay categorías sugeridas.
- **Integración con pacman:** obtiene la descripción y la versión oficiales del paquete.
- **AppImages:** los instala en `~/Apps` con permisos `700` y los ejecuta por su nombre desde el catálogo.
- **Sin dependencias extra:** solo necesita `pacman` (y `sudo` para instalar paquetes).

## 📋 Requisitos

- **Arch Linux** o una distribución con `pacman`.
- **Rust 1.85 o superior** para compilar (el proyecto usa la edición 2024).
- **`sudo`** para instalar paquetes con `pkg install`.

## 🛠️ Instalación

El binario se llama **`pkg`**. Compílalo desde el directorio del proyecto:

```bash
cargo build --release
sudo install -Dm755 target/release/pkg /usr/local/bin/pkg
```

También puedes instalarlo solo para tu usuario con Cargo:

```bash
cargo install --path .
```

Comprueba que funciona:

```bash
pkg --version
pkg --help
```

## 🚀 Primeros pasos

```bash
# Ver el catálogo (también puedes ejecutar `pkg` sin subcomando)
pkg list

# Instalar un paquete con pacman y registrarlo
pkg install htop -c System

# Buscar una herramienta por nombre, descripción o categoría
pkg search network
```

## 📚 Comandos

| Comando   | Alias           | Descripción                                                                 |
| :-------- | :-------------- | :-------------------------------------------------------------------------- |
| `add`     | —               | Registra en el catálogo un paquete **ya instalado** (no instala nada).      |
| `install` | `i`             | Instala con pacman y lo registra; si recibe un `.AppImage`, lo instala en `~/Apps` y lo registra. |
| `list`    | `ls`            | Lista los paquetes del catálogo.                                            |
| `search`  | `-q`, `--query` | Busca por nombre, descripción, categoría, comando u origen.                 |
| `info`    | —               | Muestra la ficha de un paquete y refresca sus metadatos desde pacman.       |
| `remove`  | —               | Elimina un paquete del catálogo (no lo desinstala del sistema).             |
| `execute` | `e`             | Ejecuta un AppImage registrado en el catálogo.                              |

### Sintaxis

```text
pkg add     [OPCIONES] [PAQUETE]...
pkg install [OPCIONES] [PAQUETE]...
pkg list
pkg search  <TEXTO>          # también: pkg -q <TEXTO> | pkg --query <TEXTO>
pkg info    [PAQUETE]...
pkg remove  [PAQUETE]...
pkg execute <PAQUETE>        # también: pkg e <PAQUETE>
```

> **Nota:** si ejecutas `pkg` sin subcomando, se muestra el catálogo (equivale a `pkg list`).

### Opciones comunes de `add` e `install`

| Opción                    | Descripción                                                                                                                   |
| :------------------------ | :---------------------------------------------------------------------------------------------------------------------------- |
| `-c, --category [CAT]`    | Categoría personal. Sin valor (`pkg add -c`) muestra las categorías en uso y las sugeridas.                                   |
| `--command <CMD>`         | Comando con el que se invoca si difiere del nombre del paquete (por ejemplo, `zoxide` → `z`). En AppImages fija el nombre en el catálogo. |
| `-d, --description <TEXT>`| Descripción personal; tiene prioridad sobre la de pacman.                                                                     |
| `-o, --origin <ORIGEN>`   | Origen del registro (`manual`, `pacman`, `appimage` o texto libre). Por defecto: `manual` en `add`, `pacman` en `install` y `appimage` en AppImages. |

`install` acepta varios paquetes a la vez. Si un paquete no existe en los repositorios, avisa y **solo lo registra** (útil para AUR u otras vías). En el caso de los AppImages, si faltan `--command` o `-d`, se preguntan de forma interactiva.

## 🧪 Ejemplos

```bash
# Registrar una herramienta ya instalada
pkg add fzf -c Terminal -d "Buscador difuso para la terminal"

# Instalar y registrar con los metadatos de pacman
pkg install htop -c System

# Registrar un comando distinto del nombre del paquete
pkg install zoxide --command z -c Terminal

# Registrar un paquete de AUR u otra vía (no se instala, solo se registra)
pkg install paru -o AUR

# Ver las categorías en uso y las sugeridas
pkg add -c
```

### AppImages

```bash
# Instala el AppImage en ~/Apps (permisos 700) y lo registra como "gimp"
pkg install ~/Descargas/GIMP-2.10.34-x86_64.AppImage \
  --command gimp -c Other -d "Editor de imágenes"

# Ejecutarlo desde el catálogo
pkg execute gimp
pkg e gimp
```

> El fichero `.AppImage` **se mueve** a `~/Apps`; no se conserva una copia en la ubicación original. Si el destino ya existe, se pide confirmación antes de sobrescribirlo. Solo se pueden ejecutar AppImages registrados, y `pkg execute` no admite argumentos adicionales para la aplicación.

### Consultar y eliminar

```bash
pkg info gimp     # ficha con descripción, versión, categoría, comando y origen
pkg remove gimp   # solo lo quita del catálogo, no desinstala nada
```

## 🗄️ Datos y almacenamiento

- **Base de datos:** `$XDG_DATA_HOME/pkg/pkg.db` (por defecto, `~/.local/share/pkg/pkg.db`). Es SQLite y se crea automáticamente.
- **AppImages:** `~/Apps`.
- **Categorías sugeridas:** `Development`, `Files`, `Git`, `System`, `Terminal`, `Network`, `Other`. Puedes usar cualquier otra.
- **Salida de `list` y `search`:** nombre, categoría, comando entre corchetes y descripción.

## 🔮 Roadmap

- ✨ Selector interactivo: un menú guiado para elegir herramientas sin recordar comandos.
- 🧩 Etiquetas además de categorías.
- 🐧 Soporte para otros gestores de paquetes más allá de pacman.

## 🤝 Contribuciones

¡Las contribuciones son bienvenidas! Puedes abrir un *issue* o un *pull request* con mejoras, correcciones o nuevas ideas para el catálogo.

---

_Proyecto personal creado para mejorar la experiencia con la línea de comandos._
