//! Apertura de la base SQLite en `%ProgramData%\SmartDisk Monitor\`, con WAL y transacciones
//! breves (constitución §V). Aplica las migraciones pendientes al abrir.

use std::path::{Path, PathBuf};

use rusqlite::Connection;

use super::migrations::{self, MigrationError};

pub const NOMBRE_FICHERO: &str = "smartdisk.sqlite";

/// Abre (o crea) la base en `data_dir`, activa WAL y deja el esquema al día.
pub fn open(data_dir: &Path) -> Result<(Connection, PathBuf), MigrationError> {
    std::fs::create_dir_all(data_dir)?;
    let db_path = data_dir.join(NOMBRE_FICHERO);

    let mut conn = Connection::open(&db_path)?;

    // WAL: lecturas concurrentes con la escritura del ciclo de recopilación, sin bloquearse mutuamente.
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", true)?;
    // Espera antes de fallar por bloqueo en vez de devolver error al primer roce (docs/ui-contract.md
    // §1, `db.locked` es retryable).
    conn.busy_timeout(std::time::Duration::from_secs(5))?;

    migrations::apply_pending(&mut conn, &db_path)?;

    // Una prueba manual que quedó `pending`/`running`/`cancelling` de una sesión anterior no
    // terminó sola: el proceso que la seguía ya no existe. Marcarla `interrupted` evita que
    // `test.busy` (`docs/open-questions.md` J.29) bloquee para siempre el disco al que apuntaba.
    let ahora = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default();
    conn.execute(
        "UPDATE test_runs SET status = 'interrupted', finished_at_utc = ?1
         WHERE status IN ('pending', 'running', 'cancelling')",
        [ahora],
    )?;

    Ok((conn, db_path))
}

/// Estado gestionado por Tauri: la conexión, única y compartida entre comandos. `Mutex` porque
/// `rusqlite::Connection` no es `Sync`; SQLite serializa igualmente sus escrituras con WAL, así
/// que el coste real del bloqueo es bajo.
pub struct AppState {
    pub conn: std::sync::Mutex<Connection>,
    pub db_path: PathBuf,
    /// `None` = monitorización activa; `Some(desde_utc)` = pausada. **Nunca se persiste**: la
    /// pausa no sobrevive a un reinicio (FR-031, US-074) — un monitor que arranca pausado y no lo
    /// dice es un monitor que no vigila en silencio.
    pub paused: std::sync::Mutex<Option<String>>,
    /// Cuándo se notificó por última vez cada grupo de alertas (id → instante), para el cooldown
    /// de notificación de `alerts::notificaciones` (`docs/alert-rules.md` "Cooldown de
    /// notificación"). **Tampoco se persiste**: reiniciar la aplicación puede como mucho volver a
    /// notificar algo que ya se vio, nunca dejar de notificar algo nuevo — el lado seguro de esa
    /// pérdida, igual que `paused`.
    pub notified_at: std::sync::Mutex<std::collections::HashMap<String, time::OffsetDateTime>>,
    /// Señal de cancelación de cada prueba manual en curso (T083), por `test_run_id`. Vive solo en
    /// memoria: no tiene sentido persistir la cancelación de un hilo que ya no existe tras un
    /// reinicio — esas filas quedan `interrupted` al arrancar, igual que cualquier trabajo que no
    /// pudo cerrarse solo.
    pub test_cancel_flags: std::sync::Mutex<
        std::collections::HashMap<String, std::sync::Arc<std::sync::atomic::AtomicBool>>,
    >,
}

impl AppState {
    pub fn open(data_dir: &Path) -> Result<Self, MigrationError> {
        let (conn, db_path) = open(data_dir)?;
        Ok(Self {
            conn: std::sync::Mutex::new(conn),
            db_path,
            paused: std::sync::Mutex::new(None),
            notified_at: std::sync::Mutex::new(std::collections::HashMap::new()),
            test_cancel_flags: std::sync::Mutex::new(std::collections::HashMap::new()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abre_con_wal_y_migraciones_aplicadas() {
        let dir = crate::test_util::temp_dir_unico("db");
        let (conn, path) = open(&dir).unwrap();

        let modo: String = conn
            .pragma_query_value(None, "journal_mode", |r| r.get(0))
            .unwrap();
        assert_eq!(modo.to_lowercase(), "wal");
        assert!(path.ends_with(NOMBRE_FICHERO));

        let version: i64 = conn
            .query_row("SELECT MAX(version) FROM schema_migrations", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(version, 2);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
