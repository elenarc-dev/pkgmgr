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

La instalación es sencilla: simplemente añade la ubicación del ejecutable a tu variable `$PATH`.

**1. Instalar el Paquete:**
*(Asumiendo que ya estás en un entorno donde `pkgmgr` puede ser instalado)*

```bash
# Ejemplo de instalación (ajusta según el método real)
sudo pacman -S pkgmgr 
```

**2. Verificar la Instalación:**
Una vez instalado, el comando principal estará disponible:

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

### 2. Añadir un Comando
Ejemplo: Quieres añadir `htop` para monitorear procesos.

```bash
pkg add htop --description "Monitor interactivo de recursos del sistema, mejor que top." --category System
```

> **Tip:** Si el paquete está instalado con `pacman`, usa `pkg install <paquete>` para registrarlo automáticamente y guardar la información más detallada.

### 3. Buscar un Comando Específico
Si necesitas saber qué hace la herramienta de redes:

```bash
pkg search networking
# o pkg -q networking
```

## 📚 Comandos Disponibles (Reference)

`PkgMgr` ha sido diseñado con los siguientes comandos subyacentes para manejar todas tus necesidades:

| Comando | Alias(s) | Descripción | Uso Recomendado |
| :--- | :--- | :--- | :--- |
| **`list`** | `ls` | Muestra todos los paquetes registrados en el catálogo. | Revisión general del inventario. |
| **`search`** | `-q`, `--query` | Busca por nombre, descripción o categoría. Es la función más usada. | Cuando recuerdas *qué hace* pero no el *nombre*. |
| **`add`** | N/A | Añade un paquete manualmente (ej. AppImage o binario sin gestor). | Para herramientas externas o manuales. |
| **`info`** | N/A | Muestra detalles completos de un comando registrado, incluyendo categorías y descripciones extendidas. | Verificar el contexto completo de una herramienta. |
| **`remove`** | N/A | Elimina el registro del catálogo (¡Advertencia! No desinstala nada en tu sistema). | Cuando ya no utilizas un programa. |
| **`execute`**| `e` | Ejecuta directamente un comando registrado por su nombre. | Para usar una herramienta de forma rápida sin escribir todo el path. |

## 🔮 Próximos Planes (Roadmap)

*   ✅ Soporte avanzado para AppImages y binarios externos.
*   ✨ Modo selector interactivo: Un menú guiado que te permite elegir herramientas sin recordar comandos.
*   🧩 Mayor soporte de categorías y etiquetas personalizadas.

---
_Este proyecto es un catálogo personal diseñado por el usuario para mejorar la experiencia con la línea de comandos._

**Contribuciones:** ¡Sé parte del esfuerzo! Ayúdanos a catalogar más comandos y mejoras.