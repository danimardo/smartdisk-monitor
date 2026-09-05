//! Ejecutor de migraciones: SQL numerado, compilado dentro del binario y nunca editado una vez
//! publicado (constitución §V). Antes de aplicar migraciones pendientes se crea una copia
//! consistente de la base y se conservan las tres más recientes.

use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::Connection;
use sha2::{Digest, Sha256};

/// Una migración compilada en el binario. `sql` se congela en tiempo de compilación con
/// `include_str!`: no hay forma de que el fichero en disco y el código diverjan.
pub struct Migration {
    pub version: i64,
    pub name: &'static str,
    pub sql: &'static str,
}

/// Lista cerrada y ordenada. Añadir una migración es añadir una entrada nueva al final; nunca se
/// edita una existente.
pub const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "esquema_inicial",
        sql: include_str!("../../migrations/0001_esquema_inicial.sql"),
    },
    Migration {
        version: 2,
        name: "agregados_metricas",
        sql: include_str!("../../migrations/0002_agregados_metricas.sql"),
    },
];

#[derive(Debug)]
pub enum MigrationError {
    Sqlite(rusqlite::Error),
    Io(std::io::Error),
    /// El checksum registrado no coincide con el de la migración compilada: alguien la editó
    /// después de publicarla, o la base viene de un binario con una migración distinta.
    ChecksumMismatch {
        version: i64,
    },
}

impl From<rusqlite::Error> for MigrationError {
    fn from(e: rusqlite::Error) -> Self {
        MigrationError::Sqlite(e)
    }
}

impl From<std::io::Error> for MigrationError {
    fn from(e: std::io::Error) -> Self {
        MigrationError::Io(e)
    }
}

fn checksum(sql: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(sql.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn now_utc_iso() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default()
}

/// Copia consistente de `db_path` antes de migrar, y purga las copias más allá de las tres
/// últimas. Usa la copia en caliente de SQLite (`VACUUM INTO`), correcta con la base abierta.
pub fn backup_before_migrate(conn: &Connection, db_path: &Path) -> Result<PathBuf, MigrationError> {
    let backups_dir = db_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("migraciones_previas");
    fs::create_dir_all(&backups_dir)?;

    use time::macros::format_description;
    let stamp = time::OffsetDateTime::now_utc()
        .format(format_description!(
            "[year][month][day][hour][minute][second]"
        ))
        .unwrap_or_else(|_| "0".repeat(14));
    let backup_path = backups_dir.join(format!("antes_de_migrar_{stamp}.sqlite"));

    conn.execute(
        "VACUUM INTO ?1",
        [backup_path.to_string_lossy().to_string()],
    )?;

    let mut copias: Vec<PathBuf> = fs::read_dir(&backups_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("sqlite"))
        .collect();
    copias.sort();
    while copias.len() > 3 {
        let mas_antigua = copias.remove(0);
        let _ = fs::remove_file(mas_antigua);
    }

    Ok(backup_path)
}

/// Aplica toda migración cuya versión sea mayor que la última registrada en `schema_migrations`.
/// Devuelve las versiones aplicadas, en orden. Idempotente: si no hay pendientes, no toca nada ni
/// crea copia — una copia por cada arranque sin cambios llenaría el disco sin motivo.
pub fn apply_pending(conn: &mut Connection, db_path: &Path) -> Result<Vec<i64>, MigrationError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at_utc TEXT NOT NULL,
            checksum TEXT NOT NULL
        )",
    )?;

    let ultima: i64 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |r| r.get(0),
    )?;

    // Toda migración ya aplicada debe seguir teniendo el mismo checksum que la compilada. Si no,
    // la base y el binario cuentan historias distintas de la misma migración.
    {
        let mut stmt = conn.prepare("SELECT version, checksum FROM schema_migrations")?;
        let filas = stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?;
        for fila in filas {
            let (version, checksum_guardado) = fila?;
            if let Some(m) = MIGRATIONS.iter().find(|m| m.version == version) {
                if checksum(m.sql) != checksum_guardado {
                    return Err(MigrationError::ChecksumMismatch { version });
                }
            }
        }
    }

    let pendientes: Vec<&Migration> = MIGRATIONS.iter().filter(|m| m.version > ultima).collect();
    if pendientes.is_empty() {
        return Ok(vec![]);
    }

    if ultima > 0 {
        backup_before_migrate(conn, db_path)?;
    }

    let mut aplicadas = Vec::new();
    for migracion in pendientes {
        let tx = conn.transaction()?;
        tx.execute_batch(migracion.sql)?;
        tx.execute(
            "INSERT INTO schema_migrations (version, applied_at_utc, checksum) VALUES (?1, ?2, ?3)",
            rusqlite::params![migracion.version, now_utc_iso(), checksum(migracion.sql)],
        )?;
        tx.commit()?;
        aplicadas.push(migracion.version);
    }

    Ok(aplicadas)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_db_path(nombre: &str) -> PathBuf {
        let dir = crate::test_util::temp_dir_unico("migraciones");
        fs::create_dir_all(&dir).unwrap();
        dir.join(nombre)
    }

    #[test]
    fn aplica_desde_cero_y_registra_la_version() {
        let db_path = temp_db_path("desde_cero.sqlite");
        let mut conn = Connection::open(&db_path).unwrap();

        let aplicadas = apply_pending(&mut conn, &db_path).unwrap();
        assert_eq!(aplicadas, vec![1, 2]);

        let cuenta: i64 = conn
            .query_row("SELECT COUNT(*) FROM schema_migrations", [], |r| r.get(0))
            .unwrap();
        assert_eq!(cuenta, 2);

        // Las doce tablas del modelo existen.
        for tabla in [
            "devices",
            "volumes",
            "device_volume_links",
            "metric_samples",
            "smart_snapshots",
            "system_events",
            "alert_groups",
            "alert_occurrences",
            "test_runs",
            "settings",
            "event_cursors",
            "schema_migrations",
        ] {
            let existe: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name = ?1",
                    [tabla],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(existe, 1, "falta la tabla {tabla}");
        }

        let existe_agregados: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name = 'metric_aggregates'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(existe_agregados, 1, "falta metric_aggregates (migración 2)");
    }

    #[test]
    fn es_idempotente() {
        let db_path = temp_db_path("idempotente.sqlite");
        let mut conn = Connection::open(&db_path).unwrap();

        apply_pending(&mut conn, &db_path).unwrap();
        let segunda_pasada = apply_pending(&mut conn, &db_path).unwrap();

        assert!(segunda_pasada.is_empty(), "no debe reaplicar nada");
    }

    #[test]
    fn detecta_una_migracion_alterada_tras_publicarse() {
        let db_path = temp_db_path("alterada.sqlite");
        let mut conn = Connection::open(&db_path).unwrap();
        apply_pending(&mut conn, &db_path).unwrap();

        // Simula que el checksum guardado ya no coincide con el de la migración compilada.
        conn.execute(
            "UPDATE schema_migrations SET checksum = 'manipulado' WHERE version = 1",
            [],
        )
        .unwrap();

        let resultado = apply_pending(&mut conn, &db_path);
        assert!(matches!(
            resultado,
            Err(MigrationError::ChecksumMismatch { version: 1 })
        ));
    }

    #[test]
    fn crea_copia_previa_y_conserva_solo_las_tres_ultimas() {
        let db_path = temp_db_path("copias.sqlite");
        let mut conn = Connection::open(&db_path).unwrap();
        apply_pending(&mut conn, &db_path).unwrap();

        let backups_dir = db_path.parent().unwrap().join("migraciones_previas");
        fs::create_dir_all(&backups_dir).unwrap();
        // Simula cuatro copias previas ya existentes; la quinta llamada debe dejar como máximo 3.
        for i in 0..4 {
            fs::write(
                backups_dir.join(format!("antes_de_migrar_2020010100000{i}.sqlite")),
                b"x",
            )
            .unwrap();
        }

        backup_before_migrate(&conn, &db_path).unwrap();

        let restantes = fs::read_dir(&backups_dir).unwrap().count();
        assert!(
            restantes <= 3,
            "deberían quedar como mucho 3 copias, hay {restantes}"
        );
    }

    /// T106: no hay todavía ninguna versión publicada anterior a esta (es la primera), así que lo
    /// verificable hoy es el mecanismo de actualización en sí, no una base real de una versión
    /// previa. Simula justo eso: una base que solo llegó a la migración 1 (como si viniera de un
    /// binario publicado antes de que existiera la 2), reabierta con el binario actual — debe
    /// aplicar solo la 2, en orden, sin volver a tocar la 1, y terminar con el mismo esquema que
    /// una instalación desde cero.
    #[test]
    fn actualiza_desde_una_version_publicada_anterior_sin_reaplicar_lo_ya_hecho() {
        let db_path = temp_db_path("actualizacion.sqlite");
        let mut conn = Connection::open(&db_path).unwrap();

        // Solo la migración 1, como dejaría una versión publicada antes de que existiera la 2.
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                applied_at_utc TEXT NOT NULL,
                checksum TEXT NOT NULL
            )",
        )
        .unwrap();
        conn.execute_batch(MIGRATIONS[0].sql).unwrap();
        conn.execute(
            "INSERT INTO schema_migrations (version, applied_at_utc, checksum) VALUES (1, ?1, ?2)",
            rusqlite::params![now_utc_iso(), checksum(MIGRATIONS[0].sql)],
        )
        .unwrap();

        // "Actualizar" con el binario actual: debe ver la 1 ya aplicada y aplicar solo la 2.
        let aplicadas = apply_pending(&mut conn, &db_path).unwrap();
        assert_eq!(
            aplicadas,
            vec![2],
            "solo debe aplicar lo pendiente, nunca reaplicar la 1"
        );

        let version_final: i64 = conn
            .query_row("SELECT MAX(version) FROM schema_migrations", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(version_final, 2);

        let existe_agregados: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name = 'metric_aggregates'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            existe_agregados, 1,
            "el esquema final debe quedar idéntico al de una instalación desde cero"
        );

        // Una actualización real (venía de una versión anterior con datos) sí debe crear copia de
        // seguridad antes de tocar el esquema — a diferencia de una instalación desde cero.
        let backups_dir = db_path.parent().unwrap().join("migraciones_previas");
        assert!(
            backups_dir.is_dir() && fs::read_dir(&backups_dir).unwrap().next().is_some(),
            "una actualización con datos previos debe dejar una copia de seguridad"
        );
    }
}
