//! Interacción con pacman: la autoridad sobre el estado real del sistema.

use std::process::Command;

use anyhow::{Context, Result, anyhow};

use crate::models::PacmanInfo;

/// Ejecuta `pacman -Qi <pkg>` y devuelve su salida completa.
/// Error claro si el paquete no está instalado o pacman falla.
pub fn query_installed_info(pkg: &str) -> Result<String> {
    let output = Command::new("pacman")
        .arg("-Qi")
        .arg(pkg)
        .output()
        .with_context(|| format!("no se pudo ejecutar pacman -Qi {pkg}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stderr = stderr.trim();
        if stderr.contains("no se encontró") || stderr.contains("was not found") {
            anyhow::bail!("'{pkg}' no está instalado en el sistema");
        }
        anyhow::bail!("pacman -Qi {pkg} falló: {stderr}");
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Extrae el valor de un campo tipo "Nombre    : valor".
fn field(info: &str, key: &str) -> Option<String> {
    info.lines().find_map(|line| {
        let (head, value) = line.split_once(':')?;
        head.trim().eq_ignore_ascii_case(key).then(|| value.trim().to_string())
    })
}

/// Convierte la salida de `pacman -Qi` en metadatos útiles para la caché.
pub fn parse_info(output: &str) -> Result<PacmanInfo> {
    let description = field(output, "Descripción")
        .or_else(|| field(output, "Description"))
        .ok_or_else(|| anyhow!("no se pudo leer la descripción de pacman -Qi"))?;
    let version = field(output, "Versión")
        .or_else(|| field(output, "Version"))
        .unwrap_or_default();

    Ok(PacmanInfo { description, version })
}

/// Consulta pacman y devuelve los metadatos ya parseados.
pub fn fetch_info(pkg: &str) -> Result<PacmanInfo> {
    parse_info(&query_installed_info(pkg)?)
}

/// Comprueba si un paquete existe en los repositorios configurados
/// (`pacman -Si`, sin sudo). No distingue repo/repos:
/// solo responde "¿existe o no?".
pub fn find_in_repos(pkg: &str) -> Result<bool> {
    let output = Command::new("pacman")
        .args(["-Si", pkg])
        .output()
        .with_context(|| format!("no se pudo ejecutar pacman -Si {pkg}"))?;

    Ok(output.status.success())
}

/// Instala un paquete con `sudo pacman -S --needed`.
///
/// El stdio se hereda: los logs de pacman se muestran en vivo, sudo puede
/// pedir la contraseña y todas las preguntas de pacman (confirmación,
/// elección de proveedores, conflictos...) se responden de forma interactiva.
/// `--needed` solo evita reinstalar si el paquete ya está instalado.
pub fn install(pkg: &str) -> Result<()> {
    let status = Command::new("sudo")
        .args(["pacman", "-S", "--needed"])
        .arg(pkg)
        .status()
        .with_context(|| format!("no se pudo ejecutar sudo pacman -S {pkg}"))?;

    if !status.success() {
        anyhow::bail!("la instalación de '{pkg}' falló (código {:?})", status.code());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "Nombre            : mise\nVersión           : 0.15.0-1\nDescripción       : The front-end to your dev env\n";

    #[test]
    fn parsea_descripcion_y_version() {
        let info = parse_info(SAMPLE).unwrap();
        assert_eq!(info.description, "The front-end to your dev env");
        assert_eq!(info.version, "0.15.0-1");
    }

    #[test]
    fn soporta_locales_en_ingles() {
        let sample = "Name            : fzf\nVersion         : 0.44.1-1\nDescription     : Command-line fuzzy finder\n";
        let info = parse_info(sample).unwrap();
        assert_eq!(info.description, "Command-line fuzzy finder");
        assert_eq!(info.version, "0.44.1-1");
    }

    #[test]
    fn falla_sin_descripcion() {
        assert!(parse_info("Nombre : x").is_err());
    }
}
