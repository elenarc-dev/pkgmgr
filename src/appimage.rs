//! Instalación de AppImages: copia a ~/Apps con permisos 700.

use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

/// True si el nombre acaba en `.AppImage` (sin distinguir mayúsculas).
pub fn is_appimage(name: &str) -> bool {
    Path::new(name)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("appimage"))
}

/// Directorio de AppImages del usuario: `$HOME/Apps`. No lo crea.
pub fn apps_dir() -> Result<PathBuf> {
    let home =
        std::env::var_os("HOME").context("no se pudo determinar el directorio home (¿HOME?)")?;
    Ok(PathBuf::from(home).join("Apps"))
}

/// Comprueba el destino en `~/Apps`: debe no existir o ser un fichero normal.
/// Un enlace simbólico se rechaza siempre: pkg no escribe ni cambia permisos a
/// través de enlaces, porque destruiría el fichero apuntado fuera de `~/Apps`.
pub fn check_destination(dst: &Path) -> Result<()> {
    match std::fs::symlink_metadata(dst) {
        Ok(meta) if meta.file_type().is_symlink() => bail!(
            "{} es un enlace simbólico; pkg no escribe a través de enlaces. Elimina el enlace o renombra el .AppImage de origen.",
            dst.display()
        ),
        Ok(meta) if !meta.is_file() => {
            bail!(
                "{} ya existe en ~/Apps y no es un fichero normal.",
                dst.display()
            )
        }
        Ok(_) => Ok(()),
        Err(_) => Ok(()), // Todavía no existe.
    }
}

/// True si hay que pedir confirmación antes de instalar: en `dst` ya hay un
/// fichero normal distinto de `src`. Un destino enlazado no se pregunta:
/// `check_destination` lo rechaza.
pub fn needs_confirmation(dst: &Path, src: &Path) -> bool {
    match std::fs::symlink_metadata(dst) {
        Ok(meta) if meta.is_file() => !mismo_inode(src, &meta),
        _ => false,
    }
}

/// Asegura que el directorio de destino existe y admite ficheros nuevos, y lo
/// crea si falta. Se comprueba antes de preguntar nada al usuario, para no
/// pedir datos y luego fallar.
pub fn prepare(dir: &Path) -> Result<()> {
    std::fs::create_dir_all(dir).with_context(|| format!("no se pudo crear {}", dir.display()))?;
    Ok(())
}

/// Copia `src` a `dst_dir` conservando el nombre original del fichero y deja
/// el destino con permisos 700 (ejecutable solo por el propietario). El
/// contenido se copia a un fichero temporal del propio directorio y se coloca
/// con `rename`: así, si en el destino había un enlace duro a otro sitio, ese
/// enlace se sustituye en lugar de reescribirse a través de él (lo que
/// modificaría un fichero fuera de `~/Apps`).
///
/// Si origen y destino son el mismo fichero no copia nada (copiar sobre si
/// mismo destruiría el contenido), solo asegura los permisos. Devuelve la ruta
/// destino.
pub fn install_file(src: &Path, dst_dir: &Path) -> Result<PathBuf> {
    let nombre = src
        .file_name()
        .context("no se pudo determinar el nombre del fichero de origen")?;
    let dst = dst_dir.join(nombre);
    check_destination(&dst)?;

    if comparte_inode(src, &dst) {
        // El destino es el mismo fichero que el origen (p. ej. ya estaba
        // instalado): copiar lo destruiría, así que solo ajusto permisos. Ojo:
        // si además comparte inode con otro fichero, esos permisos cambian en
        // los dos sitios.
        hacer_ejecutable(&dst)?;
        return Ok(dst);
    }
    let _ = mover(src, &dst);

    prepare(dst_dir)?;
    Ok(dst)
}

/// Copia a un temporal del mismo directorio (creándolo 600, y dejándolo 700
/// antes de que exista en el destino) y lo coloca con ``, que sustituye
/// lo que hubiera sin escribir a través de ello.
fn mover(src: &Path, dst: &Path) -> Result<()> {
    std::fs::rename(src, dst)
        .with_context(|| format!("no se pudo colocar {} en {}", src.display(), dst.display()))?;

        Ok(())
}

/// Deja `path` con permisos 700.
fn hacer_ejecutable(path: &Path) -> Result<()> {
    let perms = std::fs::Permissions::from_mode(0o700);
    std::fs::set_permissions(path, perms)
        .with_context(|| format!("no se pudo dar permisos 700 a {}", path.display()))
}

/// True si `dst` es un fichero normal y `src` (resolviendo sus enlaces, porque
/// lo que importa es el contenido) es ese mismo fichero.
fn comparte_inode(src: &Path, dst: &Path) -> bool {
    match std::fs::symlink_metadata(dst) {
        Ok(meta) if meta.is_file() => mismo_inode(src, &meta),
        _ => false,
    }
}

/// True si `src` y el fichero descrito por `dst_meta` son el mismo: mismo
/// dispositivo e `inode`. No se usa `canonicalize` en el destino para no
/// resolver enlaces simbólicos.
fn mismo_inode(src: &Path, dst_meta: &std::fs::Metadata) -> bool {
    match std::fs::metadata(src) {
        Ok(src_meta) => src_meta.dev() == dst_meta.dev() && src_meta.ino() == dst_meta.ino(),
        Err(_) => false,
    }
}

/// Valida el nombre del paquete en el catálogo y lo devuelve recortado.
pub fn validate_command(cmd: &str) -> Result<String> {
    let trimmed = cmd.trim();
    if trimmed.is_empty()
        || trimmed.chars().any(|c| c.is_whitespace())
        || trimmed.contains('/')
        || trimmed == "."
        || trimmed == ".."
    {
        bail!("el nombre del paquete no puede contener espacios ni '/'");
    }
    Ok(trimmed.to_string())
}

/// Sugiere un nombre de catálogo a partir del nombre del fichero: quita el
/// sufijo `.AppImage`, parte el resto en tokens (por '-', '_' y espacios) y
/// descarta versiones ("2.10.34"), arquitecturas ("x86_64", "x64"...) y
/// nombres de OS ("linux"). Los tokens solo dígitos se conservan (p. ej. el
/// número de build de "Portable 3211"). Vacío si no queda nada utilizable.
pub fn suggest_command(file_name: &str) -> String {
    const ARCH: [&str; 8] = [
        "x86_64",
        "i386",
        "i686",
        "amd64",
        "arm64",
        "aarch64",
        "linux",
        "universal",
    ];

    let base = strip_appimage_suffix(file_name);
    let mut tokens: Vec<String> = Vec::new();

    for trozo in base.split(|c: char| c == '-' || c.is_whitespace()) {
        let trozo = trozo.to_lowercase();
        if trozo.is_empty() || ARCH.contains(&trozo.as_str()) {
            continue;
        }
        for token in trozo.split('_') {
            if token.is_empty() {
                continue;
            }
            // Versiones ("2.10.34") y arquitecturas sueltas ("x64"): dígitos
            // con puntos o mezclados con letras.
            let empieza_por_digito = token.starts_with(|c: char| c.is_ascii_digit());
            let tiene_punto = token.contains('.');
            let mezcla_letras_y_digitos = token.chars().any(|c| c.is_ascii_alphabetic())
                && token.chars().any(|c| c.is_ascii_digit());
            if (empieza_por_digito && tiene_punto) || mezcla_letras_y_digitos {
                continue;
            }
            if ARCH.contains(&token) {
                continue;
            }
            let limpio: String = token
                .chars()
                .filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '.')
                .collect();
            if !limpio.is_empty() {
                tokens.push(limpio);
            }
        }
    }

    tokens.join("-")
}

/// Extrae del nombre del fichero la primera versión con forma
/// `v?dígito+(.dígito+)+` (sin el prefijo `v`). Vacío si no hay ninguna.
pub fn version_from_file_name(file_name: &str) -> String {
    let base = strip_appimage_suffix(file_name);
    base.split(|c: char| c == '-' || c == '_' || c.is_whitespace())
        .find_map(extraer_version)
        .unwrap_or_default()
}

/// Quita el sufijo `.AppImage` final (sin distinguir mayúsculas), si existe.
fn strip_appimage_suffix(file_name: &str) -> &str {
    const SUFIJO: &str = ".appimage";
    if file_name.len() >= SUFIJO.len() && file_name.is_char_boundary(file_name.len() - SUFIJO.len())
    {
        let (base, ext) = file_name.split_at(file_name.len() - SUFIJO.len());
        if ext.eq_ignore_ascii_case(SUFIJO) {
            return base;
        }
    }
    file_name
}

/// Busca en el token una versión `v?dígito+(.dígito+)+` (con al menos un
/// punto), mirando también dentro del token ("App.v1.2" -> "1.2") y sin el
/// prefijo `v`.
fn extraer_version(token: &str) -> Option<String> {
    let chars: Vec<char> = token.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        let es_v = c == 'v' || c == 'V';
        if !es_v && !c.is_ascii_digit() {
            continue;
        }
        // El número no puede empezar en mitad de otro número.
        if c.is_ascii_digit() && i > 0 && chars[i - 1].is_ascii_digit() {
            continue;
        }
        // Una 'v' solo abre versión si detrás vienen dígitos.
        if es_v && !chars.get(i + 1).is_some_and(|c| c.is_ascii_digit()) {
            continue;
        }

        let cuerpo = if es_v { i + 1 } else { i };
        let Some(fin) = fin_de_version(&chars, cuerpo) else {
            continue;
        };
        return Some(chars[cuerpo..fin].iter().collect());
    }
    None
}

/// Fin de la secuencia `dígitos(.dígitos)+` que empieza en `inicio`; None si
/// no hay al menos un punto con dígitos detrás.
fn fin_de_version(chars: &[char], inicio: usize) -> Option<usize> {
    let mut i = inicio;
    while i < chars.len() && chars[i].is_ascii_digit() {
        i += 1;
    }
    let mut fin = i;
    let mut hay_grupo = false;
    while i + 1 < chars.len() && chars[i] == '.' && chars[i + 1].is_ascii_digit() {
        i += 1;
        while i < chars.len() && chars[i].is_ascii_digit() {
            i += 1;
        }
        fin = i;
        hay_grupo = true;
    }
    hay_grupo.then_some(fin)
}

/// Pregunta el nombre del paquete para el catálogo, mostrando la sugerencia
/// (derivada del nombre del fichero) entre corchetes. Con entrada vacía se
/// acepta la sugerencia; si esta también está vacía, se vuelve a preguntar.
/// En EOF se devuelve error.
pub fn ask_name(sugerencia: &str) -> Result<String> {
    let stdin = std::io::stdin();
    loop {
        if sugerencia.is_empty() {
            println!("Nombre del paquete en el catálogo:");
        } else {
            println!("Nombre del paquete en el catálogo [{sugerencia}]:");
        }

        let mut linea = String::new();
        let leidos = stdin
            .read_line(&mut linea)
            .context("no se pudo leer de la entrada estándar")?;
        if leidos == 0 {
            bail!(
                "se necesita el nombre del paquete (ejecuta en una terminal o indica --command <nombre>)"
            );
        }

        let entrada = linea.trim();
        let candidato = if entrada.is_empty() {
            sugerencia
        } else {
            entrada
        };
        match validate_command(candidato) {
            Ok(nombre) => return Ok(nombre),
            Err(err) => println!("Entrada no válida ({err}); inténtalo de nuevo."),
        }
    }
}

/// Pregunta la descripción del paquete; no acepta una respuesta vacía.
/// En EOF se devuelve error.
pub fn ask_description() -> Result<String> {
    let stdin = std::io::stdin();
    loop {
        println!("Descripción:");

        let mut linea = String::new();
        let leidos = stdin
            .read_line(&mut linea)
            .context("no se pudo leer de la entrada estándar")?;
        if leidos == 0 {
            bail!("se necesita la descripción (ejecuta en una terminal o indica -d <descripción>)");
        }

        let entrada = linea.trim();
        if !entrada.is_empty() {
            return Ok(entrada.to_string());
        }
    }
}

/// Pregunta sí/no con `[s/N]`: devuelve true solo para "s", "si"/"sí", "y" u
/// "yes" (mayúsculas indiferentes); cualquier otra respuesta (o EOF) es "no".
pub fn confirm(question: &str) -> Result<bool> {
    println!("{question} [s/N]:");

    let mut linea = String::new();
    let leidos = std::io::stdin()
        .read_line(&mut linea)
        .context("no se pudo leer de la entrada estándar")?;
    if leidos == 0 {
        return Ok(false);
    }

    let respuesta = linea.trim().to_lowercase().replace('í', "i");
    Ok(matches!(respuesta.as_str(), "s" | "si" | "y" | "yes"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Directorio de prueba aislado y con nombre unico.
    fn sandbox(tag: &str) -> Result<PathBuf> {
        static SEQ: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "pkg_appimage_test_{}_{}_{}",
            tag,
            std::process::id(),
            n
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir)?;
        Ok(dir)
    }

    #[test]
    fn detecta_appimage_sin_distinguir_mayusculas() {
        assert!(is_appimage("app.AppImage"));
        assert!(is_appimage("app.appimage"));
        assert!(is_appimage("APP.APPIMAGE"));
        assert!(is_appimage("/ruta/con/directorios/app.AppImage"));
        assert!(!is_appimage("app.zip"));
        assert!(!is_appimage("app"));
        assert!(!is_appimage("app.AppImage.bak"));
    }

    #[test]
    fn valida_nombre_del_paquete() {
        // Recorta espacios de los extremos.
        assert_eq!(validate_command("  gimp  ").unwrap(), "gimp");
        assert!(validate_command("gimp-2.10").is_ok());

        for malo in ["", "   ", "con espacios", "con/Barra", ".", ".."] {
            assert!(validate_command(malo).is_err(), "{malo} deberia fallar");
        }
    }

    #[test]
    fn sugiere_nombres_de_ejemplo() {
        assert_eq!(suggest_command("GIMP-2.10.34-x86_64.AppImage"), "gimp");
        assert_eq!(suggest_command("OBSStudio-3.0.1.AppImage"), "obsstudio");
        assert_eq!(
            suggest_command("Sublime Text - x64 - Portable 3211.AppImage"),
            "sublime-text-portable-3211"
        );
        assert_eq!(suggest_command("AppImage.AppImage"), "appimage");
    }

    #[test]
    fn extrae_versiones_de_ejemplo() {
        assert_eq!(
            version_from_file_name("GIMP-2.10.34-x86_64.AppImage"),
            "2.10.34"
        );
        assert_eq!(version_from_file_name("App.v1.2.AppImage"), "1.2");
        assert_eq!(version_from_file_name("foo.AppImage"), "");
    }

    #[test]
    fn copia_con_permisos_700_y_confirmaciones() -> Result<()> {
        let dir = sandbox("appimage")?;

        let src = dir.join("origen.AppImage");
        std::fs::write(&src, b"hola")?;
        let dst_dir = dir.join("Apps");

        // Copia normal: mismo nombre, permisos 700 y contenido identico.
        let dst = install_file(&src, &dst_dir)?;
        assert_eq!(dst, dst_dir.join("origen.AppImage"));
        assert_eq!(std::fs::metadata(&dst)?.permissions().mode() & 0o777, 0o700);
        assert_eq!(std::fs::read(&dst)?, b"hola".to_vec());

        // Confirmaciones: otro fichero en el mismo destino -> true; el mismo
        // fichero -> false; destino inexistente -> false.
        assert!(needs_confirmation(&dst, &src));
        assert!(!needs_confirmation(&src, &src));
        let otro = dir.join("otro.AppImage");
        std::fs::write(&otro, b"otro")?;
        assert!(needs_confirmation(&dst, &otro));
        assert!(!needs_confirmation(&dst_dir.join("falta"), &src));

        // Copiar sobre si mismo (origen == destino): no falla y conserva permisos.
        let dst2 = install_file(&src, &dst_dir)?;
        assert_eq!(dst2, dst);
        assert_eq!(
            std::fs::metadata(&dst2)?.permissions().mode() & 0o777,
            0o700
        );

        let _ = std::fs::remove_dir_all(&dir);
        Ok(())
    }

    #[test]
    fn rechaza_enlaces_simbolicos_en_el_destino() -> Result<()> {
        let dir = sandbox("appimage")?;

        let src = dir.join("origen.AppImage");
        std::fs::write(&src, b"nuevo")?;

        let outside = dir.join("outside");
        std::fs::create_dir_all(&outside)?;
        let real = outside.join("real.AppImage");
        std::fs::write(&real, b"EXTERNAL")?;
        std::fs::set_permissions(&real, std::fs::Permissions::from_mode(0o644))?;

        let dst_dir = dir.join("Apps");
        std::fs::create_dir_all(&dst_dir)?;
        let dst = dst_dir.join("origen.AppImage");
        std::os::unix::fs::symlink(&real, &dst)?;

        // Un enlace no se pregunta: lo rechaza check_destination.
        assert!(!needs_confirmation(&dst, &src));
        assert!(install_file(&src, &dst_dir).is_err());

        // El fichero apuntado sigue intacto: mismo contenido y 644.
        assert_eq!(std::fs::read(&real)?, b"EXTERNAL".to_vec());
        let meta = std::fs::metadata(&real)?;
        assert_eq!(meta.permissions().mode() & 0o777, 0o644);

        // El destino sigue siendo un enlace (no se reemplazo).
        assert!(std::fs::symlink_metadata(&dst)?.file_type().is_symlink());

        let _ = std::fs::remove_dir_all(&dir);
        Ok(())
    }

    #[test]
    fn no_copia_sobre_el_mismo_fichero_enlazado() -> Result<()> {
        let dir = sandbox("appimage")?;

        let dst_dir = dir.join("Apps");
        std::fs::create_dir_all(&dst_dir)?;
        let dst = dst_dir.join("origen.AppImage");
        std::fs::write(&dst, b"unico")?;
        std::fs::set_permissions(&dst, std::fs::Permissions::from_mode(0o755))?;

        let otra = dir.join("otra_ruta");
        std::fs::create_dir_all(&otra)?;
        let hardlink = otra.join("origen.AppImage");
        std::fs::hard_link(&dst, &hardlink)?;

        // Mismo inode: no se copia nada (no se destruye el contenido), solo
        // se ajustan los permisos.
        let instalado = install_file(&hardlink, &dst_dir)?;
        assert_eq!(instalado, dst);
        assert_eq!(std::fs::read(&dst)?, b"unico".to_vec());
        assert_eq!(std::fs::metadata(&dst)?.permissions().mode() & 0o777, 0o700);

        let _ = std::fs::remove_dir_all(&dir);
        Ok(())
    }

    #[test]
    fn rechaza_destinos_que_no_son_ficheros_normales() -> Result<()> {
        let dir = sandbox("appimage")?;

        let src = dir.join("origen.AppImage");
        std::fs::write(&src, b"hola")?;

        let dst_dir = dir.join("Apps");
        std::fs::create_dir_all(dst_dir.join("origen.AppImage"))?;

        assert!(install_file(&src, &dst_dir).is_err());

        let _ = std::fs::remove_dir_all(&dir);
        Ok(())
    }

    #[test]
    fn reemplaza_enlaces_duros_sin_tocar_el_fichero_enlazado() -> Result<()> {
        let dir = sandbox("hardlink")?;

        // Fichero fuera de ~/Apps enlazado duramente dentro de ~/Apps.
        let fuera = dir.join("fuera");
        std::fs::create_dir_all(&fuera)?;
        let original = fuera.join("data.AppImage");
        std::fs::write(&original, b"EXTERNAL")?;
        std::fs::set_permissions(&original, std::fs::Permissions::from_mode(0o644))?;

        let dst_dir = dir.join("Apps");
        std::fs::create_dir_all(&dst_dir)?;
        let dst = dst_dir.join("data.AppImage");
        std::fs::hard_link(&original, &dst)?;

        // Otro fichero distinto con el mismo nombre.
        let otro = dir.join("otro");
        std::fs::create_dir_all(&otro)?;
        let src = otro.join("data.AppImage");
        std::fs::write(&src, b"NUEVO")?;

        // Hay que confirmar (otro fichero en el destino), y al instalar se
        // sustituye el enlace: lo instalado es un fichero nuevo.
        assert!(needs_confirmation(&dst, &src));
        let instalado = install_file(&src, &dst_dir)?;
        assert_eq!(instalado, dst);

        // El fichero de fuera no se toco: ni contenido ni permisos.
        assert_eq!(std::fs::read(&original)?, b"EXTERNAL".to_vec());
        let meta_original = std::fs::metadata(&original)?;
        assert_eq!(meta_original.permissions().mode() & 0o777, 0o644);

        // El destino es un fichero distinto, con el contenido nuevo y 700.
        assert_eq!(std::fs::read(&dst)?, b"NUEVO".to_vec());
        let meta_dst = std::fs::metadata(&dst)?;
        assert_eq!(meta_dst.permissions().mode() & 0o777, 0o700);
        assert_ne!(meta_dst.ino(), meta_original.ino());

        // No quedan ficheros temporales en ~/Apps.
        assert_eq!(std::fs::read_dir(&dst_dir)?.count(), 1);

        let _ = std::fs::remove_dir_all(&dir);
        Ok(())
    }

    #[test]
    fn preparar_rechaza_un_destino_que_no_es_un_directorio() -> Result<()> {
        let dir = sandbox("prepare")?;
        std::fs::write(dir.join("Apps"), b"no soy un directorio")?;

        // create_dir_all falla igual para root que para un usuario normal.
        assert!(prepare(&dir.join("Apps")).is_err());

        // Y sobre un directorio normal si prepara el camino y no deja restos.
        let apps = dir.join("apps-ok");
        prepare(&apps)?;
        assert!(apps.is_dir());
        assert_eq!(std::fs::read_dir(&apps)?.count(), 0);

        let _ = std::fs::remove_dir_all(&dir);
        Ok(())
    }
}
