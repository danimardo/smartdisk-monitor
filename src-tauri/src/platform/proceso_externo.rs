//! Lanzar un proceso externo sin dos problemas que comparten solución porque los sufre cualquier
//! proceso externo que este backend lanza (`smartctl`, PowerShell para inventario y para Control de
//! acceso a carpetas): que se cuelgue para siempre (J.55) y que parpadee una ventana de consola
//! visible aunque la aplicación no tenga terminal propia (J.57).

use std::io::{self, Read};
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Sin esto, cualquier proceso de consola lanzado desde una aplicación GUI hace parpadear una
/// ventana negra aunque termine en milisegundos: Windows le asigna una consola nueva al lanzarlo,
/// antes de que el propio proceso (p. ej. `-WindowStyle Hidden` de PowerShell) decida nada sobre su
/// estilo — ese modificador llega demasiado tarde para evitar la ventana en sí.
#[cfg(windows)]
fn sin_ventana(cmd: &mut Command) {
    use std::os::windows::process::CommandExt;
    cmd.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
fn sin_ventana(_cmd: &mut Command) {}

fn spawn_con_pipes(mut cmd: Command) -> io::Result<Child> {
    sin_ventana(&mut cmd);
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()
}

/// Ejecuta `cmd` con un límite de tiempo, sin depender de un crate de temporización externo:
/// `Child` no tiene una espera con plazo en la biblioteca estándar.
///
/// Los pipes de `stdout`/`stderr` tienen un búfer acotado por el sistema operativo. Si nadie los
/// vacía mientras el hijo sigue escribiendo, el hijo se bloquea en su propio `write()` en cuanto lo
/// llena, y nunca llega a salir — un punto muerto de facto entre el hijo y este bucle si se leyeran
/// los pipes **después** de que `try_wait()` confirmara la salida (J.55, medido contra hardware
/// real: la tabla de atributos SMART completa de un SATA supera ese búfer con facilidad y colgaba
/// el intento entero). Los hilos lectores vacían los pipes según llegan, sin esperar a que el
/// proceso termine.
///
/// `Ok(None)` = se agotó el límite y se mató al proceso; quien llama decide si eso cuenta como
/// fallo o como dato ausente — igual que cualquier otro dato que no llegó a tiempo.
pub fn ejecutar_con_limite(cmd: Command, limite: Duration) -> io::Result<Option<Output>> {
    ejecutar_con_limite_cancelable(cmd, limite, || false)
}

/// Como [`ejecutar_con_limite`], pero además consulta `debe_parar()` en cada vuelta del bucle de
/// espera: si devuelve `true` se mata el proceso y se devuelve `Ok(None)` (la misma señal que el
/// agotamiento del límite). Para la prueba de Rendimiento (ADR-053): cancelación del usuario y
/// guardia térmica matan la invocación de DiskSpd en curso.
pub fn ejecutar_con_limite_cancelable(
    cmd: Command,
    limite: Duration,
    debe_parar: impl Fn() -> bool,
) -> io::Result<Option<Output>> {
    let mut hijo = spawn_con_pipes(cmd)?;

    let mut stdout = hijo.stdout.take();
    let mut stderr = hijo.stderr.take();
    let lector_stdout = std::thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(s) = stdout.as_mut() {
            let _ = s.read_to_end(&mut buf);
        }
        buf
    });
    let lector_stderr = std::thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(s) = stderr.as_mut() {
            let _ = s.read_to_end(&mut buf);
        }
        buf
    });

    let inicio = Instant::now();
    loop {
        if let Some(estado) = hijo.try_wait()? {
            let salida = lector_stdout.join().unwrap_or_default();
            let error = lector_stderr.join().unwrap_or_default();
            return Ok(Some(Output {
                status: estado,
                stdout: salida,
                stderr: error,
            }));
        }
        if inicio.elapsed() >= limite || debe_parar() {
            let _ = hijo.kill();
            let _ = hijo.wait();
            let _ = lector_stdout.join();
            let _ = lector_stderr.join();
            return Ok(None);
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(windows)]
    fn ejecutar_con_limite_no_se_bloquea_con_una_salida_mas_grande_que_el_pipe() {
        // Regresión (J.55): antes se leían los pipes solo después de que `try_wait()` confirmara
        // la salida del proceso. Con una salida mayor que el búfer del pipe, el hijo se bloqueaba
        // escribiendo y nunca llegaba a salir — se agotaba el límite entero, indistinguible de un
        // proceso que de verdad no responde. No hace falta ningún binario concreto para
        // reproducirlo: cualquier proceso que escriba lo bastante lo dispara.
        let mut cmd = std::process::Command::new("cmd");
        cmd.args([
            "/C",
            "for /L %i in (1,1,4000) do @echo 0123456789012345678901234567890123456789",
        ]);

        let resultado = ejecutar_con_limite(cmd, Duration::from_secs(10)).unwrap();
        let salida = resultado.expect("no debería agotar el límite de tiempo");
        assert!(salida.status.success());
        assert!(
            salida.stdout.len() > 64 * 1024,
            "la salida capturada debería superar el búfer típico de un pipe, midió {} bytes",
            salida.stdout.len()
        );
    }

    #[test]
    #[cfg(windows)]
    fn ejecutar_con_limite_mata_un_proceso_que_no_termina_solo() {
        // `ping -t` no termina nunca por su cuenta: es el sustituto más simple de "un proceso
        // colgado de verdad" que no depende de ningún binario propio del proyecto.
        let mut cmd = std::process::Command::new("ping");
        cmd.args(["-t", "127.0.0.1"]);

        let resultado = ejecutar_con_limite(cmd, Duration::from_millis(300)).unwrap();
        assert!(
            resultado.is_none(),
            "debería agotar el límite, no esperar para siempre"
        );
    }
}
