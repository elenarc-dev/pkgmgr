//! Definición de la CLI con clap.

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "pkg",
    version,
    about = "Catálogo personal de herramientas CLI para Arch Linux",
    after_help = "Sin subcomando: selector interactivo (próximamente)."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Añade un paquete instalado al catálogo personal (no instala nada).
    Add {
        /// Nombre del paquete según pacman
        package: Option<Vec<String>>,
        /// Categoría personal; sin valor, muestra las categorías disponibles
        #[arg(short, long, num_args = 0..=1, default_missing_value = "")]
        category: Option<String>,
        /// Comando a usar si difiere del nombre del paquete (ej. zoxide -> z)
        #[arg(long, value_name = "CMD")]
        command: Option<String>,
        /// Descripción personal; sobrescribe la anterior si el paquete ya existía
        #[arg(short = 'd', long, value_name = "TEXT")]
        description: Option<String>,
        /// Origen del paquete (por defecto "manual")
        #[arg(short = 'o', long, value_name = "ORIGIN")]
        origin: Option<String>,
    },
    /// Instala con pacman y registra en el catálogo; si <paquete> es un fichero
    /// .AppImage lo copia a ~/Apps (permisos 700) y lo registra (--command es
    /// el nombre en el catálogo y, si faltan --command o -d, se preguntan de
    /// forma interactiva). Si el paquete no existe en los repos avisa pero lo
    /// registra igual (útil para AUR u otras vías).
    #[command(visible_alias = "i")]
    Install {
        /// Nombre del paquete según pacman, o fichero .AppImage (con o sin ruta)
        package: Option<Vec<String>>,
        /// Categoría personal; sin valor, muestra las categorías disponibles
        #[arg(short, long, num_args = 0..=1, default_missing_value = "")]
        category: Option<String>,
        /// Comando a usar si difiere del nombre del paquete (ej. zoxide -> z)
        #[arg(long, value_name = "CMD")]
        command: Option<String>,
        /// Descripción personal; sobrescribe la de pacman si el paquete ya existe
        #[arg(short = 'd', long, value_name = "TEXT")]
        description: Option<String>,
        /// Origen del paquete (por defecto "pacman" en install)
        #[arg(short = 'o', long, value_name = "ORIGIN")]
        origin: Option<String>,
    },
    /// Lista los paquetes del catálogo.
    #[command(visible_alias = "ls")]
    List,
    /// Busca por nombre, descripción, categoría o comando.
    #[command(short_flag = 'q', long_flag = "query")]
    Search {
        /// Texto a buscar (insensible a mayúsculas)
        query: String,
    },
    /// Muestra información de un paquete del catálogo.
    Info {
        /// Nombre del paquete en el catálogo
        package: Vec<String>,
    },
    /// Elimina un paquete del catálogo (no desinstala nada).
    Remove {
        /// Nombre del paquete en el catálogo
        package: Vec<String>,
    },
    /// Ejecuta un AppImage registrado en el catálogo por su nombre.
    #[command(visible_alias = "e")]
    Execute {
        /// Nombre del paquete en el catálogo
        package: String,
    },
}
