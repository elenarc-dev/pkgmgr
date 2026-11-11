mod appimage;
mod cli;
mod database;
mod models;
mod pacman;

use anyhow::{Context, Result, bail};
use rusqlite::Connection;

use clap::Parser;
use cli::{Cli, Command};
use models::{PacmanInfo, Pkg};

fn main() {
    let cli = Cli::parse();

    if let Err(err) = run(cli) {
        eprintln!("Error: {err:#}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Some(Command::Add {
            package,
            category,
            command,
            description,
            origin,
        }) => cmd_add(
            package.as_deref(),
            category,
            command.as_deref(),
            description.as_deref(),
            origin.as_deref(),
        ),
        Some(Command::Install {
            package,
            category,
            command,
            description,
            origin,
        }) => cmd_install(
            package.as_deref(),
            category,
            command.as_deref(),
            description.as_deref(),
            origin.as_deref(),
        ),
        Some(Command::List) => cmd_list(),
        Some(Command::Search { query }) => cmd_search(&query),
        Some(Command::Info { package }) => cmd_info(package),
        Some(Command::Remove { package }) => cmd_remove(package),
        Some(Command::Execute { package }) => cmd_execute(&package),
        None => {
            // eprintln!("Uso: pkg <COMANDO>\n\nEjecuta 'pkg --help' para ver los comandos disponibles.");
            // Ok(())
            cmd_list()
        }
    }
}

/// `pkg add [<paquete>] [-c <categoria>] [--command <cmd>] [-d <desc>] [-o <origen>]`:
/// valida con pacman y guarda en la BD. Con `-c` sin valor muestra las
/// categorías disponibles.
fn cmd_add(
    package: Option<&[String]>,
    category: Option<String>,
    command: Option<&str>,
    description: Option<&str>,
    origin: Option<&str>,
) -> Result<()> {
    if category.as_deref() == Some("") {
        let conn = database::open()?;
        return show_categories(&conn);
    }
    let Some(names) = package else {
        bail!(
            "se requiere <paquete>. Uso: pkg add <paquete> [-c <categoría>] [-d <desc>] [-o <origen>]"
        );
    };
    for name in names {
        let info = pacman::fetch_info(name.as_str()).unwrap_or_default(); // falla con error claro si no está instalado

        let conn = database::open()?;
        let existed = database::get_info(&conn, name)?.is_some();
        database::upsert(
            &conn,
            name,
            category.as_deref(),
            command,
            origin,
            description,
            None,
            &info,
        )?;

        match (existed, category.as_deref()) {
            (true, Some(cat)) => println!("'{name}' actualizado (categoría: {cat})."),
            (true, None) => println!("'{name}' ya estaba en el catálogo; metadatos actualizados."),
            (false, Some(cat)) => println!("Añadido '{name}' al catálogo (categoría: {cat})."),
            (false, None) => println!("Añadido '{name}' al catálogo."),
        }
    }
    Ok(())
}

/// `pkg install [<paquete>] [-c <categoria>] [--command <cmd>] [-d <desc>] [-o <origen>]`
/// tiene dos modos:
///
/// - Paquetes (por defecto): instala con pacman (logs visibles) y registra en
///   el catálogo con origen "pacman" por defecto. Si el paquete no existe en
///   los repos lo avisa pero lo registra igual (útil para AUR u otras vías,
///   p. ej. `pkg install paquete -o AUR`).
/// - AppImages: cualquier <paquete> que acabe en `.AppImage` (con o sin ruta)
///   se copia a `~/Apps` con permisos 700 y se registra en el catálogo;
///   `--command` fija el nombre en el catálogo y `-d` la descripción (si
///   faltan, se preguntan de forma interactiva).
///
/// Con `-c` sin valor muestra las categorías disponibles.
fn cmd_install(
    package: Option<&[String]>,
    category: Option<String>,
    command: Option<&str>,
    description: Option<&str>,
    origin: Option<&str>,
) -> Result<()> {
    if category.as_deref() == Some("") {
        let conn = database::open()?;
        return show_categories(&conn);
    }
    let Some(names) = package else {
        bail!(
            "se requiere <paquete>. Uso: pkg install <paquete> [-c <categoría>] [-d <desc>] [-o <origen>] (alias: pkg i)"
        );
    };

    // Cada <paquete> es un paquete pacman o un fichero .AppImage; se reparten
    // por rutas distintas.
    let (appimages, pacman_refs): (Vec<&String>, Vec<&String>) = names
        .iter()
        .partition(|n| appimage::is_appimage(n.as_str()));

    if appimages.is_empty() && origin.is_some_and(|x| x.eq_ignore_ascii_case("appimage")) {
        bail!(
            "para instalar un AppImage indica el fichero .AppImage como <paquete>: pkg install <app.AppImage> --command <nombre> [-d <descripción>] [-c <categoría>]"
        );
    }
    if appimages.len() > 1 && (command.is_some() || description.is_some()) {
        bail!(
            "con varios .AppImage el nombre y la descripción se preguntan por cada fichero; no pueden fijarse con --command ni -d"
        );
    }

    if !pacman_refs.is_empty() {
        let pacman_names: Vec<String> = pacman_refs.iter().map(|s| (*s).clone()).collect();
        install_pacman(
            category.as_deref(),
            command,
            description,
            origin,
            &pacman_names,
        )?;
    }
    for file in appimages {
        install_appimage(
            file.as_str(),
            category.as_deref(),
            command,
            description,
            origin,
        )?;
    }
    Ok(())
}

/// Instala un fichero .AppImage: copia a `~/Apps` (permisos 700) y registro en
/// el catálogo. Primero reúne todos los datos (nombre via `--command` o
/// preguntando, descripción via `-d` o preguntando, confirmaciones de
/// sobrescritura del registro y del fichero destino) y solo entonces copia.
fn install_appimage(
    file: &str,
    category: Option<&str>,
    command: Option<&str>,
    description: Option<&str>,
    origin: Option<&str>,
) -> Result<()> {
    let src = std::path::Path::new(file);
    if !src.is_file() {
        bail!("'{file}' no es un fichero .AppImage accesible");
    }
    let file_name = src
        .file_name()
        .context("no se pudo determinar el nombre del fichero")?
        .to_string_lossy()
        .into_owned();
    let apps = appimage::apps_dir()?;
    let dst = apps.join(&file_name);
    appimage::check_destination(&dst)?;
    // Compruebo que puedo crear ficheros en ~/Apps antes de pedir datos.
    appimage::prepare(&apps)?;

    // Nombre en el catálogo: flag --command o pregunta interactiva.
    let name = match command {
        Some(cmd) => appimage::validate_command(cmd)?,
        None => appimage::ask_name(&appimage::suggest_command(&file_name))?,
    };

    // Descripción: flag -d o pregunta interactiva.
    let desc = match description.map(str::trim).filter(|d| !d.is_empty()) {
        Some(d) => d.to_string(),
        None => appimage::ask_description()?,
    };

    // Sobrescritura del registro en el catálogo.
    let conn = database::open()?;
    if database::get_info(&conn, &name)?.is_some()
        && !appimage::confirm(&format!("'{name}' ya está en el catálogo. ¿Sobrescribir?"))?
    {
        bail!("instalación cancelada: '{name}' ya existe en el catálogo");
    }

    // Sobrescritura del fichero en ~/Apps.
    let dst_display = dst.display().to_string();
    if appimage::needs_confirmation(&dst, src)
        && !appimage::confirm(&format!("'{dst_display}' ya existe. ¿Sobrescribir?"))?
    {
        bail!(
            "instalación cancelada: {dst_display} ya existe (renombra el .AppImage de origen si quieres conservar ambos)"
        );
    }

    let installed = appimage::install_file(src, &apps)?;
    println!(
        "AppImage instalado en {} (permisos 700).",
        installed.display()
    );

    // La descripción se guarda en los dos campos: en el de pacman (lo que
    // consulta `pkg search`) y como descripción personal (así sobrevive a un
    // futuro `pkg add`/`pkg info`, que reescriben la caché de pacman con datos
    // vacíos para un AppImage). La versión sale del nombre del fichero.
    let info = PacmanInfo {
        description: desc.clone(),
        version: appimage::version_from_file_name(&file_name),
    };
    cmd_register(
        &name,
        &info,
        category,
        Some(name.as_str()),
        Some(desc.as_str()),
        Some(origin.unwrap_or("appimage")),
        Some(&file_name),
    )
}

fn install_pacman(
    category: Option<&str>,
    command: Option<&str>,
    description: Option<&str>,
    origin: Option<&str>,
    names: &[String],
) -> Result<(), anyhow::Error> {
    for name in names {
        match pacman::find_in_repos(name) {
            Ok(true) => pacman::install(name)?,
            Ok(false) => {
                println!(
                    "Aviso: '{name}' no existe en los repositorios de pacman.\nNo se instala nada, pero se registra en el catálogo igualmente."
                );
            }
            Err(e) => return Err(e),
        }

        // Si está instalado por otra vía (AUR, makepkg...) pacman da sus
        // metadatos; si no está instalado en absoluto (p. ej. un typo) se
        // registra con metadatos vacíos, como pide el comportamiento de
        // "avisar pero añadir igual".
        let info = pacman::fetch_info(name).unwrap_or_default();
        cmd_register(name, &info, category, command, description, origin, None)?;
    }
    Ok(())
}

/// Registra un paquete en el catálogo con origen por defecto "pacman"
/// (a diferencia de `cmd_add`, que usa "manual").
fn cmd_register(
    name: &str,
    info: &PacmanInfo,
    category: Option<&str>,
    command: Option<&str>,
    description: Option<&str>,
    origin: Option<&str>,
    appimage_file: Option<&str>,
) -> Result<()> {
    let origin = origin.or(Some("pacman"));
    let conn = database::open()?;
    let existed = database::get_info(&conn, name)?.is_some();
    database::upsert(&conn, name, category, command, origin, description, appimage_file, info)?;

    match (existed, category) {
        (true, Some(cat)) => println!("'{name}' actualizado (categoría: {cat})."),
        (true, None) => println!("'{name}' ya estaba en el catálogo; metadatos actualizados."),
        (false, Some(cat)) => println!("Añadido '{name}' al catálogo (categoría: {cat})."),
        (false, None) => println!("Añadido '{name}' al catálogo."),
    }
    Ok(())
}

/// Muestra las categorías en uso (con conteo) y las sugeridas.
fn show_categories(conn: &Connection) -> Result<()> {
    let in_use = database::list_categories(conn)?;

    println!("Categorías en uso:");
    if in_use.is_empty() {
        println!("  (ninguna todavía)");
    } else {
        for (cat, count) in &in_use {
            println!("  {cat:<12} {count}");
        }
    }

    let in_use_lc: Vec<String> = in_use.iter().map(|(c, _)| c.to_lowercase()).collect();
    println!("\nSugeridas:");
    for cat in models::SUGGESTED_CATEGORIES {
        if !in_use_lc.iter().any(|u| *u == cat.to_lowercase()) {
            println!("  {cat}");
        }
    }
    Ok(())
}

/// `pkg list`: muestra el catálogo personal (solo lo que tú añadiste).
fn cmd_list() -> Result<()> {
    let conn = database::open()?;
    print_packages(&conn, &database::list(&conn)?)
}

/// `pkg search <texto>` (o `-q`): busca en nombre, descripción, categoría y comando.
fn cmd_search(query: &str) -> Result<()> {
    let conn = database::open()?;
    let results = database::search(&conn, query)?;

    if results.is_empty() {
        bail!("sin resultados para '{query}' en tu catálogo.");
    }
    print_packages(&conn, &results)
}

/// Imprime paquetes en formato columna (usado por list y search).
fn print_packages(conn: &Connection, packages: &[Pkg]) -> Result<()> {
    let width = packages.iter().map(|p| p.name.len()).max().unwrap_or(0);
    for p in packages {
        let description = database::get_description(conn, &p.name)?.unwrap_or_default();
        let cat = p.category.as_deref().unwrap_or("-");
        let cmd = p
            .command
            .as_deref()
            .map(|c| format!("[{c}]"))
            .unwrap_or_default();
        println!(
            "{:<width$}  {:<10}  {:<6}  {}",
            p.name,
            cat,
            cmd,
            description,
            width = width
        );
    }
    Ok(())
}

/// `pkg info <paquete>`: refresca desde pacman y muestra la ficha.
fn cmd_info(names: Vec<String>) -> Result<()> {
    let conn = database::open()?;

    for name in names {
        let cached = database::get_info(&conn, name.as_str())?;
        if cached.is_none() {
            bail!("'{name}' no está en tu catálogo. Añádelo con 'pkg add {name}'.");
        }

        // pacman es la autoridad: se consulta en vivo y se refresca la caché.
        // Si el paquete no está instalado (se puede registrar sin instalarlo),
        // se muestra lo que haya en caché con un aviso.
        let info = match pacman::fetch_info(name.as_str()) {
            Ok(info) => {
                database::upsert(&conn, name.as_str(), None, None, None, None, None, &info)?;
                info
            }
            Err(_) => {
                println!(
                    "(aviso: '{name}' no está instalado en el sistema; se muestran los datos en caché)"
                );
                cached.unwrap()
            }
        };

        println!("Paquete      : {name}");
        let description =
            database::get_description(&conn, name.as_str())?.unwrap_or(info.description);
        println!(
            "Descripción  : {}",
            if description.is_empty() {
                "-"
            } else {
                &description
            }
        );
        println!("Versión      : {}", info.version);

        let pkg = database::list(&conn)?.into_iter().find(|p| p.name == name);
        if let Some(p) = pkg {
            if let Some(cat) = p.category {
                println!("Categoría    : {cat}");
            }
            if let Some(cmd) = p.command {
                println!("Comando      : {cmd}");
            }
            if let Some(origin) = p.origin {
                println!("Origen       : {origin}");
            }
            println!("Añadido el   : {}", p.added_at);
        }
    }
    Ok(())
}

/// `pkg remove <paquete>`: saca el paquete del catálogo (no desinstala).
fn cmd_remove(names: Vec<String>) -> Result<()> {
    let conn = database::open()?;
    for name in names {
        if database::remove(&conn, name.as_str())? {
            println!("'{name}' eliminado del catálogo.");
        } else {
            bail!("'{name}' no está en tu catálogo.");
        }
    }
    Ok(())
}

/// `pkg execute <paquete>` (alias `e`): ejecuta un AppImage registrado en el catálogo.
fn cmd_execute(name: &str) -> Result<()> {
    let conn = database::open()?;

    // Verificamos que el paquete esté en el catálogo.
    let Some(pkg) = database::list(&conn)?
        .into_iter()
        .find(|p| p.name == *name)
    else {
        bail!("'{name}' no está en tu catálogo. Añádelo con 'pkg install <archivo.AppImage>'.");
    };

    // Solo ejecutamos AppImages.
    if pkg.origin.as_deref() != Some("appimage") {
        bail!("'{name}' no es un AppImage (origen: {:?}). Solo se pueden ejecutar AppImages.", pkg.origin.as_deref().unwrap_or("desconocido"));
    }

    // Usamos el nombre original del fichero guardado al instalar.
    let appimage_file = match &pkg.appimage_file {
        Some(f) => f,
        None => bail!("'{name}' no tiene registrado el nombre del fichero .AppImage. Reinstálalo con 'pkg install <archivo.AppImage>'."),
    };

    // Construimos la ruta del .AppImage en ~/Apps.
    let apps = appimage::apps_dir()?;
    let appimage_path = apps.join(appimage_file);

    if !appimage_path.exists() {
        bail!(
            "No se encontró el fichero '{}'.\nVerifica que el AppImage esté instalado en ~/Apps.",
            appimage_path.display()
        );
    }

    println!("Ejecutando '{}' desde {}...", name, appimage_path.display());
    let status = std::process::Command::new(&appimage_path)
        .status()
        .with_context(|| format!("no se pudo ejecutar '{}'", appimage_path.display()))?;

    if !status.success() {
        if let Some(code) = status.code() {
            bail!("'{name}' terminó con código de salida {}", code);
        } else {
            bail!("'{name}' terminó por una señal.");
        }
    }

    Ok(())
}
