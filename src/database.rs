//! Base de datos personal (SQLite) del catálogo.

use std::path::PathBuf;

use anyhow::{Context, Result};
use rusqlite::Connection;

use crate::models::{PacmanInfo, Pkg};

/// Ruta de la base de datos: `$XDG_DATA_HOME/pkg/pkg.db` (por defecto `~/.local/share/pkg/pkg.db`).
pub fn db_path() -> Result<PathBuf> {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")))
        .context("no se pudo determinar el directorio de datos (¿HOME?)")?;

    Ok(base.join("pkg").join("pkg.db"))
}

/// Abre la base de datos, creando directorios y tablas si hace falta.
pub fn open() -> Result<Connection> {
    let path = db_path()?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)
            .with_context(|| format!("no se pudo crear {}", dir.display()))?;
    }

    let conn = Connection::open(&path)
        .with_context(|| format!("no se pudo abrir {}", path.display()))?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS packages (
            name        TEXT PRIMARY KEY,
            description TEXT NOT NULL DEFAULT '',
            version     TEXT NOT NULL DEFAULT '',
            category    TEXT,
            command     TEXT,
            added_at    TEXT NOT NULL
        );",
    )
    .context("no se pudo inicializar el esquema")?;

    // Migraciones: añadir columnas si faltan (v1->v2 command, v3 origin,
    // v4 description_custom).
    for (column, ddl) in [
        ("command", "ALTER TABLE packages ADD COLUMN command TEXT;"),
        ("origin", "ALTER TABLE packages ADD COLUMN origin TEXT;"),
        (
            "description_custom",
            "ALTER TABLE packages ADD COLUMN description_custom TEXT;",
        ),
    ] {
        let has_column: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('packages') WHERE name = ?1",
                [column],
                |row| row.get::<_, i64>(0),
            )
            .map(|n: i64| n > 0)
            .context("no se pudo inspeccionar el esquema")?;
        if !has_column {
            conn.execute_batch(ddl).with_context(|| {
                format!("no se pudo migrar la base de datos (añadir columna '{column}')")
            })?;
        }
    }

    Ok(conn)
}

/// Inserta o actualiza un paquete en el catálogo y guarda los metadatos de pacman.
///
/// - `category`/`command`/`origin`/`custom_description` solo se escriben si se
///   pasan (re-añadir sin ellos conserva lo que ya había).
/// - Sin `origin`, la primera inserción usa "manual".
/// - `description` guarda siempre la de pacman; `custom_description` (-d) es la
///   personal, que gana al mostrar (ver `get_description`).
#[allow(clippy::too_many_arguments)]
pub fn upsert(
    conn: &Connection,
    name: &str,
    category: Option<&str>,
    command: Option<&str>,
    origin: Option<&str>,
    custom_description: Option<&str>,
    info: &PacmanInfo,
) -> Result<()> {
    conn.execute(
        "INSERT INTO packages (name, description, version, category, command, origin, description_custom, added_at)
         VALUES (?1, ?2, ?3, ?4, ?5, COALESCE(?6, 'manual'), ?7, datetime('now'))
         ON CONFLICT(name) DO UPDATE SET
            description        = excluded.description,
            version            = excluded.version,
            category           = COALESCE(?4, packages.category),
            command            = COALESCE(?5, packages.command),
            origin             = COALESCE(?6, packages.origin),
            description_custom = COALESCE(?7, packages.description_custom);",
        rusqlite::params![
            name,
            info.description,
            info.version,
            category,
            command,
            origin,
            custom_description
        ],
    )
    .with_context(|| format!("no se pudo guardar '{name}'"))?;
    Ok(())
}

/// Elimina un paquete del catálogo. Devuelve false si no estaba registrado.
pub fn remove(conn: &Connection, name: &str) -> Result<bool> {
    let n = conn
        .execute("DELETE FROM packages WHERE name = ?1", [name])
        .with_context(|| format!("no se pudo eliminar '{name}'"))?;
    Ok(n > 0)
}

/// Todos los paquetes del catálogo, ordenados alfabéticamente.
pub fn list(conn: &Connection) -> Result<Vec<Pkg>> {
    let mut stmt = conn
        .prepare("SELECT name, category, command, origin, added_at FROM packages ORDER BY name")
        .context("no se pudo leer el catálogo")?;

    let rows = stmt
        .query_map([], |row| {
            Ok(Pkg {
                name: row.get(0)?,
                category: row.get(1)?,
                command: row.get(2)?,
                origin: row.get(3)?,
                added_at: row.get(4)?,
            })
        })
        .context("no se pudo leer el catálogo")?;

    rows.collect::<std::result::Result<Vec<_>, _>>()
        .context("no se pudo leer el catálogo")
}

/// Escapa los comodines de LIKE (% _) y el carácter de escape en el texto del usuario.
fn escape_like(input: &str) -> String {
    input
        .chars()
        .flat_map(|c| matches!(c, '%' | '_' | '\\').then_some('\\').into_iter().chain(std::iter::once(c)))
        .collect()
}

/// Busca paquetes cuyo nombre, descripción, categoría o comando contenga el
/// texto dado (insensible a mayúsculas para ASCII, vía LIKE de SQLite).
pub fn search(conn: &Connection, query: &str) -> Result<Vec<Pkg>> {
    let pattern = format!("%{}%", escape_like(query));
    let mut stmt = conn
        .prepare(
            "SELECT name, category, command, origin, added_at FROM packages
             WHERE name        LIKE ?1 ESCAPE '\\'
                OR description LIKE ?1 ESCAPE '\\'
                OR category    LIKE ?1 ESCAPE '\\'
                OR command     LIKE ?1 ESCAPE '\\'
                OR origin      LIKE ?1 ESCAPE '\\'
             ORDER BY name",
        )
        .context("no se pudo buscar en el catálogo")?;

    let rows = stmt
        .query_map([&pattern], |row| {
            Ok(Pkg {
                name: row.get(0)?,
                category: row.get(1)?,
                command: row.get(2)?,
                origin: row.get(3)?,
                added_at: row.get(4)?,
            })
        })
        .context("no se pudo buscar en el catálogo")?;

    rows.collect::<std::result::Result<Vec<_>, _>>()
        .context("no se pudo buscar en el catálogo")
}

/// Categorías en uso con su número de paquetes, ordenadas alfabéticamente (sin distinguir mayúsculas).
pub fn list_categories(conn: &Connection) -> Result<Vec<(String, i64)>> {
    let mut stmt = conn
        .prepare(
            "SELECT category, COUNT(*) FROM packages
             WHERE category IS NOT NULL AND category != ''
             GROUP BY category ORDER BY category COLLATE NOCASE",
        )
        .context("no se pudieron consultar las categorías")?;

    let rows = stmt
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)))
        .context("no se pudieron consultar las categorías")?;

    rows.collect::<std::result::Result<Vec<_>, _>>()
        .context("no se pudieron consultar las categorías")
}

/// Metadatos en caché de un paquete (pueden estar desactualizados; pacman manda).
pub fn get_info(conn: &Connection, name: &str) -> Result<Option<PacmanInfo>> {
    let mut stmt = conn
        .prepare("SELECT description, version FROM packages WHERE name = ?1")
        .context("no se pudo consultar la base de datos")?;

    let mut rows = stmt.query_map([name], |row| {
        Ok(PacmanInfo {
            description: row.get(0)?,
            version: row.get(1)?,
        })
    })?;

    match rows.next() {
        Some(row) => Ok(Some(row.context("error leyendo metadatos")?)),
        None => Ok(None),
    }
}

/// Descripción a mostrar de un paquete: la personal (`-d/--description`) si
/// existe, y si no la que pacman guardó en caché. None si no está registrado.
pub fn get_description(conn: &Connection, name: &str) -> Result<Option<String>> {
    let res = conn.query_row(
        "SELECT COALESCE(NULLIF(description_custom, ''), description)
         FROM packages WHERE name = ?1",
        [name],
        |row| row.get::<_, String>(0),
    );

    match res {
        Ok(description) => Ok(Some(description)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(anyhow::Error::from(e).context("no se pudo consultar la descripción")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::PacmanInfo;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE packages (
                name        TEXT PRIMARY KEY,
                description TEXT NOT NULL DEFAULT '',
                version     TEXT NOT NULL DEFAULT '',
                category    TEXT,
                command     TEXT,
                origin      TEXT,
                description_custom TEXT,
                added_at    TEXT NOT NULL
            );",
        )
        .unwrap();
        conn
    }

    #[test]
    fn lista_categorias_en_uso_con_conteo() {
        let conn = setup();
        upsert(&conn, "fzf", Some("Files"), None, None, None, &PacmanInfo::default()).unwrap();
        upsert(&conn, "mise", Some("development"), None, None, None, &PacmanInfo::default()).unwrap();
        upsert(&conn, "yazi", Some("Files"), None, None, None, &PacmanInfo::default()).unwrap();

        let cats = list_categories(&conn).unwrap();
        assert_eq!(cats, vec![("development".into(), 1), ("Files".into(), 2)]);
    }

    #[test]
    fn ignora_categorias_vacias() {
        let conn = setup();
        upsert(&conn, "fzf", Some(""), None, None, None, &PacmanInfo::default()).unwrap();
        upsert(&conn, "mise", None, None, None, None, &PacmanInfo::default()).unwrap();

        assert!(list_categories(&conn).unwrap().is_empty());
    }

    #[test]
    fn busca_por_nombre_y_descripcion() {
        let conn = setup();
        upsert(&conn, "mise", None, None, None, None, &PacmanInfo { description: "Gestor de versiones de Node, Python y Go".into(), version: String::new() }).unwrap();
        upsert(&conn, "yazi", None, None, None, None, &PacmanInfo { description: "Explorador de archivos".into(), version: String::new() }).unwrap();

        let res = search(&conn, "node").unwrap();
        assert_eq!(res.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(), ["mise"]);

        let res = search(&conn, "explorador").unwrap();
        assert_eq!(res.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(), ["yazi"]);
    }

    #[test]
    fn busca_por_comando_y_escapa_comodines() {
        let conn = setup();
        upsert(&conn, "zoxide", None, Some("z"), None, None, &PacmanInfo::default()).unwrap();
        upsert(&conn, "otro", None, Some("cien_por_ciento"), None, None, &PacmanInfo::default()).unwrap();

        let res = search(&conn, "z").unwrap();
        assert_eq!(res.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(), ["zoxide"]);

        // Un '_' literal no debe actuar como comodín.
        let res = search(&conn, "cien_por_ciento").unwrap();
        assert_eq!(res.len(), 1);
        let res = search(&conn, "cienXporXciento").unwrap();
        assert!(res.is_empty());
    }

    #[test]
    fn guarda_y_conserva_el_comando() {
        let conn = setup();
        upsert(&conn, "zoxide", Some("Terminal"), Some("z"), None, None, &PacmanInfo::default()).unwrap();
        upsert(&conn, "fzf", None, None, None, None, &PacmanInfo::default()).unwrap();

        let pkgs = list(&conn).unwrap();
        let z = pkgs.iter().find(|p| p.name == "zoxide").unwrap();
        assert_eq!(z.command.as_deref(), Some("z"));

        // Re-add sin --command no debe borrar el valor guardado.
        upsert(&conn, "zoxide", None, None, None, None, &PacmanInfo::default()).unwrap();
        let z = list(&conn).unwrap().into_iter().find(|p| p.name == "zoxide").unwrap();
        assert_eq!(z.command.as_deref(), Some("z"));
    }

    #[test]
    fn la_descripcion_personal_gana_y_el_origen_se_conserva() {
        let conn = setup();
        upsert(
            &conn,
            "fzf",
            None,
            None,
            None,
            None,
            &PacmanInfo { description: "fuzzy finder".into(), version: String::new() },
        )
        .unwrap();

        // Primera inserción sin -o: origen por defecto "manual".
        let fzf = list(&conn).unwrap().into_iter().find(|p| p.name == "fzf").unwrap();
        assert_eq!(fzf.origin.as_deref(), Some("manual"));
        assert_eq!(get_description(&conn, "fzf").unwrap().as_deref(), Some("fuzzy finder"));

        // -d sobrescribe lo que se muestra; -o fija el origen.
        upsert(
            &conn,
            "fzf",
            None,
            None,
            Some("AUR"),
            Some("Buscador difuso"),
            &PacmanInfo { description: "fuzzy finder v2".into(), version: String::new() },
        )
        .unwrap();
        assert_eq!(get_description(&conn, "fzf").unwrap().as_deref(), Some("Buscador difuso"));
        let fzf = list(&conn).unwrap().into_iter().find(|p| p.name == "fzf").unwrap();
        assert_eq!(fzf.origin.as_deref(), Some("AUR"));

        // Re-añadir sin -o ni -d conserva el origen y la descripción personal.
        upsert(&conn, "fzf", None, None, None, None, &PacmanInfo::default()).unwrap();
        let fzf = list(&conn).unwrap().into_iter().find(|p| p.name == "fzf").unwrap();
        assert_eq!(fzf.origin.as_deref(), Some("AUR"));
        assert_eq!(get_description(&conn, "fzf").unwrap().as_deref(), Some("Buscador difuso"));

        // Un origen explícito (como el "pacman" de install) sí sobrescribe.
        upsert(&conn, "fzf", None, None, Some("pacman"), None, &PacmanInfo::default()).unwrap();
        let fzf = list(&conn).unwrap().into_iter().find(|p| p.name == "fzf").unwrap();
        assert_eq!(fzf.origin.as_deref(), Some("pacman"));
    }
}
