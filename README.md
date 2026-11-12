# 🚀 PkgMgr: El Catálogo Definitivo de Comandos CLI para Linux

[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Arch Linux Compatibility](https://img.shields.io/badge/Compatible-Arch%20Linux-brightgreen.svg)]()

> ¿Estás abrumado por la cantidad de comandos CLI que tienes instalados en tu sistema? `PkgMgr` es un gestor de catálogo personal diseñado para **inventariar, organizar y recordar** todos tus utilitarios de línea de comandos de Linux, haciendo que la experiencia del terminal sea más accesible e intuitiva.

---

## ✨ ¿Qué Problema Resuelve PkgMgr?

En un entorno Linux moderno como Arch, es fácil instalar docenas de herramientas CLI (desde `git` hasta microservicios especializados). Con el tiempo, recordar el comando exacto o saber qué utilidad hace un ejecutable puede ser una tarea tediosa.

`PkgMgr` resuelve esto creando una **base de datos centralizada y navegable** de todos tus comandos favoritos. En lugar de buscar en Google o recordar nombres complejos, simplemente usas `pkg` para consultar, listar o ejecutar lo que necesitas.

## 💡 Características Principales

*   **Inventario Centralizado:** Mantén un registro único de absolutamente todos los programas CLI que utilizas.
*   **Búsqueda Inteligente:** Busca por nombre del comando, descripción o incluso categorías personalizadas.
*   **Categorización Personal:** Asigna descripciones detalladas y clasifica herramientas en categorías relevantes (ej. `Desarrollo`, `Redes`, `Utilidades`).
*   **Ejecución Directa:** No solo catálogo; también permite ejecutar directamente los AppImage o binarios registrados.
*   **Evolucionable:** Diseñado para ser el centro de control de tu flujo de trabajo CLI, con soporte futuro para más formatos (más allá de Arch/Pacman).

## 🛠️ Instalación y Configuración

La instalación es simple y requiere solo descargar el ejecutable.

**1. Instalar el Paquete:**
*(Asumiendo que ya estás en un entorno donde `pkgmgr` puede ser instalado)*

```bash
# Descarga e instala el binario manualmente, luego añádelo a tu PATH.
# Ejemplo: tar -xzf pkgmgr-vX.Y.tar.gz && sudo mv pkgmgr /usr/local/bin
```

**2. Verificar la Instalación:**
Una vez colocado en el `$PATH`, el comando principal estará disponible:

```bash
pkg --help
```

> **Nota Importante:** El ejecutable se llama `pkgmgr`, pero para simplificar tu flujo de trabajo, puedes usarlo como el alias **`pkg`**.

## 🚀 Primeros Pasos (Getting Started)

¡Usa estos comandos para empezar a catalogar tus herramientas!

### 1. Listar Paquetes Conocidos
Para ver qué has registrado hasta ahora:

```bash
pkg list
# o su alias: pkg ls
```

### 2. Añadir un Comando (Método Recomendado: `install`)
Ejemplo avanzado: Registrar un paquete y personalizar su nombre o descripción con información oficial del sistema.

```bash
# Sintaxis general: pkg install <paquete> --command <CMD_DESEADO> --description "<MIS_DETALLES>" [--category <CAT>]
pkg install htop --command topmon --description "Monitor interactivo de recursos, mejor que top." --category System
```

**Nota:** El comando `install` es el método preferido porque registra la información oficial del paquete (categoría por defecto 'System', descripción y etc.), pero permite sobreescribir detalles con parámetros adicionales.

### 3. Buscar un Comando Específico
Si necesitas saber qué hace la herramienta de redes:

```bash
pkg search networking
# o pkg -q networking
```

## 📚 Comandos Disponibles (Reference)

`PkgMgr` ha sido diseñado con los siguientes comandos subyacentes para manejar todas tus necesidades:

| Command | Alias(s) | Description | Recommended Use Case |
| :--- | :--- | :--- | :--- |
| **`install`** | `i` | Instala con pacman y registra en el catálogo. Es el método preferido, ya que extrae la información oficial del paquete. Puede ser personalizado con flags como `--command <CMD>` o `--description "<DESCRIPCIÓN>"`. | Usar siempre que se conozca el nombre del paquete (ej. \`pkg install htop\`). Ideal para capturar metadatos completos y personalizarlos. |
| **`list`** | `ls` | Muestra todos los paquetes registrados en el catálogo. | Revisión general del inventario. |

## 🔮 Próximos Planes (Roadmap)

*   ✅ Soporte avanzado para AppImages y binarios externos.
*   ✨ Modo selector interactivo: Un menú guiado que te permite elegir herramientas sin recordar comandos.
*   🧩 Mayor soporte de categorías y etiquetas personalizadas.

---
_Este proyecto es un catálogo personal diseñado por el usuario para mejorar la experiencia con la línea de comandos._

**Contribuciones:** ¡Sé parte del esfuerzo! Ayúdanos a catalogar más comandos y mejoras.