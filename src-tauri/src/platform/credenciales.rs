//! Guarda la clave de API de OpenRouter en el Administrador de credenciales de Windows (spec
//! `005-explicacion-ia`, principio XVI, FR-004). **Nunca** en SQLite, en un fichero ni en
//! `localStorage`.
//!
//! FFI a mano contra `advapi32`, siguiendo el patrón de `platform/energia.rs`: el proyecto no usa
//! el crate `windows` (solo `windows-registry`), así que esta pieza tampoco añade dependencia.
//!
//! El blob se guarda como UTF-8: solo lo lee esta misma función, no hay convención externa que
//! respetar. El proceso corre elevado (ADR-004), así que en producción la persistencia es
//! `LOCAL_MACHINE` (cifrada por DPAPI con la clave de la máquina); las pruebas usan `SESSION`
//! (efímera, sin exigir elevación).

use crate::error::{AppError, AppResult};

/// Nombre del objeto en el almacén. Estable: `guardar` lo sustituye, `borrar` lo elimina.
const TARGET: &str = "SmartDisk Monitor/OpenRouter";

/// Ámbito de persistencia de la credencial.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredPersist {
    /// Solo esta sesión de inicio de Windows. Para pruebas.
    Session,
    /// Todas las sesiones de esta máquina. Para producción (proceso elevado).
    LocalMachine,
}

/// Guarda (o sustituye) la clave.
pub fn guardar(clave: &str, persist: CredPersist) -> AppResult<()> {
    guardar_en(TARGET, clave, persist)
}

/// La clave guardada, o `None` si no hay ninguna.
pub fn leer() -> Option<String> {
    leer_de(TARGET)
}

/// Borra la credencial. Idempotente: que no exista no es error.
pub fn borrar() -> AppResult<()> {
    borrar_de(TARGET)
}

// ---------------------------------------------------------------------------- Windows

#[cfg(windows)]
mod win {
    use super::{AppError, AppResult, CredPersist};

    const CRED_TYPE_GENERIC: u32 = 1;
    const CRED_PERSIST_SESSION: u32 = 1;
    const CRED_PERSIST_LOCAL_MACHINE: u32 = 2;
    const ERROR_NOT_FOUND: i32 = 1168;

    #[repr(C)]
    struct Filetime {
        dw_low_date_time: u32,
        dw_high_date_time: u32,
    }

    #[repr(C)]
    struct CredentialW {
        flags: u32,
        typ: u32,
        target_name: *mut u16,
        comment: *mut u16,
        last_written: Filetime,
        credential_blob_size: u32,
        credential_blob: *mut u8,
        persist: u32,
        attribute_count: u32,
        attributes: *mut core::ffi::c_void,
        target_alias: *mut u16,
        user_name: *mut u16,
    }

    #[link(name = "advapi32")]
    extern "system" {
        fn CredWriteW(credential: *const CredentialW, flags: u32) -> i32;
        fn CredReadW(
            target_name: *const u16,
            typ: u32,
            flags: u32,
            credential: *mut *mut CredentialW,
        ) -> i32;
        fn CredFree(buffer: *mut core::ffi::c_void);
        fn CredDeleteW(target_name: *const u16, typ: u32, flags: u32) -> i32;
    }

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    fn persist_raw(p: CredPersist) -> u32 {
        match p {
            CredPersist::Session => CRED_PERSIST_SESSION,
            CredPersist::LocalMachine => CRED_PERSIST_LOCAL_MACHINE,
        }
    }

    fn error_almacen(que: &str) -> Box<AppError> {
        Box::new(
            AppError::new("ia.credential_store", "error.ia.credentialStore")
                .with_detail(format!("{que}: {}", std::io::Error::last_os_error())),
        )
    }

    pub(super) fn guardar_en(target: &str, clave: &str, persist: CredPersist) -> AppResult<()> {
        let target_w = wide(target);
        let mut blob = clave.as_bytes().to_vec();
        let cred = CredentialW {
            flags: 0,
            typ: CRED_TYPE_GENERIC,
            target_name: target_w.as_ptr() as *mut u16,
            comment: std::ptr::null_mut(),
            last_written: Filetime {
                dw_low_date_time: 0,
                dw_high_date_time: 0,
            },
            credential_blob_size: blob.len() as u32,
            credential_blob: blob.as_mut_ptr(),
            persist: persist_raw(persist),
            attribute_count: 0,
            attributes: std::ptr::null_mut(),
            target_alias: std::ptr::null_mut(),
            user_name: std::ptr::null_mut(),
        };
        // SAFETY: `target_w` y `blob` viven hasta el final de la llamada; los campos de puntero no
        // usados van a nulo, que `CredWriteW` documenta como válido para `CRED_TYPE_GENERIC`.
        let ok = unsafe { CredWriteW(&cred, 0) };
        if ok == 0 {
            return Err(error_almacen("CredWriteW"));
        }
        Ok(())
    }

    pub(super) fn leer_de(target: &str) -> Option<String> {
        let target_w = wide(target);
        let mut pcred: *mut CredentialW = std::ptr::null_mut();
        // SAFETY: `target_w` termina en NUL; `pcred` recibe un puntero que se libera con `CredFree`.
        let ok = unsafe { CredReadW(target_w.as_ptr(), CRED_TYPE_GENERIC, 0, &mut pcred) };
        if ok == 0 || pcred.is_null() {
            return None;
        }
        // SAFETY: `CredReadW` devolvió éxito → `pcred` apunta a un `CredentialW` cuyo blob mide
        // `credential_blob_size` bytes.
        let resultado = unsafe {
            let cred = &*pcred;
            if cred.credential_blob.is_null() || cred.credential_blob_size == 0 {
                None
            } else {
                let bytes = std::slice::from_raw_parts(
                    cred.credential_blob,
                    cred.credential_blob_size as usize,
                );
                String::from_utf8(bytes.to_vec()).ok()
            }
        };
        // SAFETY: `pcred` lo asignó `CredReadW` y no se ha liberado antes.
        unsafe { CredFree(pcred as *mut core::ffi::c_void) };
        resultado
    }

    pub(super) fn borrar_de(target: &str) -> AppResult<()> {
        let target_w = wide(target);
        // SAFETY: `target_w` termina en NUL.
        let ok = unsafe { CredDeleteW(target_w.as_ptr(), CRED_TYPE_GENERIC, 0) };
        if ok == 0 {
            if std::io::Error::last_os_error().raw_os_error() == Some(ERROR_NOT_FOUND) {
                return Ok(());
            }
            return Err(error_almacen("CredDeleteW"));
        }
        Ok(())
    }
}

#[cfg(windows)]
use win::{borrar_de, guardar_en, leer_de};

// ---------------------------------------------------------------------------- no-Windows (compila, no funcional)

#[cfg(not(windows))]
fn guardar_en(_target: &str, _clave: &str, _persist: CredPersist) -> AppResult<()> {
    Err(Box::new(AppError::new(
        "ia.credential_store",
        "error.ia.credentialStore",
    )))
}

#[cfg(not(windows))]
fn leer_de(_target: &str) -> Option<String> {
    None
}

#[cfg(not(windows))]
fn borrar_de(_target: &str) -> AppResult<()> {
    Ok(())
}

// ---------------------------------------------------------------------------- pruebas

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    /// `TargetName` único por ejecución para no chocar con otra corrida ni con la credencial real.
    fn target_de_prueba() -> String {
        let n = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("SmartDisk Monitor/test-{n}")
    }

    struct Limpieza(String);
    impl Drop for Limpieza {
        fn drop(&mut self) {
            let _ = borrar_de(&self.0);
        }
    }

    #[test]
    fn guardar_leer_borrar_ida_y_vuelta() {
        let target = target_de_prueba();
        let _limpieza = Limpieza(target.clone());

        assert_eq!(
            leer_de(&target),
            None,
            "no debería haber credencial al empezar"
        );

        guardar_en(&target, "sk-or-v1-abc123", CredPersist::Session).unwrap();
        assert_eq!(leer_de(&target).as_deref(), Some("sk-or-v1-abc123"));

        // Sustituir una existente.
        guardar_en(&target, "sk-or-v1-nueva", CredPersist::Session).unwrap();
        assert_eq!(leer_de(&target).as_deref(), Some("sk-or-v1-nueva"));

        borrar_de(&target).unwrap();
        assert_eq!(leer_de(&target), None);
    }

    #[test]
    fn borrar_una_credencial_inexistente_es_ok() {
        let target = target_de_prueba();
        assert!(borrar_de(&target).is_ok());
    }
}
