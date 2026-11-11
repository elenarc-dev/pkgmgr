/// Un paquete registrado en el catálogo personal.
#[derive(Debug, Clone)]
pub struct Pkg {
    pub name: String,
    /// Categoría personal, opcional.
    pub category: Option<String>,
    /// Ejecutable a usar si difiere del nombre del paquete (ej. zoxide -> z).
    pub command: Option<String>,
    /// De dónde salió el registro: "pacman" (via pkg install), "manual" (via
    /// pkg add) o un texto libre del usuario (-o/--origin).
    pub origin: Option<String>,
    /// Nombre original del fichero .AppImage cuando el origen es "appimage".
    pub appimage_file: Option<String>,
    /// Fecha (UTC) en que se añadió al catálogo, ISO-8601.
    pub added_at: String,
}

/// Datos de "instalado" guardados solo como caché; la autoridad siempre es pacman.
#[derive(Debug, Clone, Default)]
pub struct PacmanInfo {
    pub description: String,
    pub version: String,
}

/// Categorías sugeridas; se pueden usar otras libremente.
pub const SUGGESTED_CATEGORIES: &[&str] = &[
    "Development",
    "Files",
    "Git",
    "System",
    "Terminal",
    "Network",
    "Other",
];
