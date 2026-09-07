//! Icono de la bandeja del sistema: color agregado, menú y clic (FR-012,
//! `docs/product-specification.md` §3, `docs/open-questions.md` B.5).
//!
//! El color es el mismo `HealthState` de toda la interfaz (`domain::salud::tray_state`, espejo
//! exacto de `trayState()` en `src/lib/design/health.ts`); no hay un tipo ni una paleta nuevos. El
//! icono se genera en memoria a partir de los mismos `--sdm-*` que usa el resto de la aplicación,
//! sin fichero `.ico` versionado: `docs/decisions.md` señala systray como pantalla aún sin revisión
//! visual completa, y esto es un primer paso, no el diseño final.
//!
//! **Forma, no solo color** (`docs/open-questions.md` J.53, constitución §VII): sobre un tile
//! redondeado del color de estado se pinta un glifo que **también** cambia con el estado —disco
//! lleno (todo en orden), disco con «!» (advertencia), disco con «×» (crítico), disco hueco (sin
//! datos), dos barras (en pausa)—, para que se distinga a 16 px y sin depender solo del color. El
//! dibujo es procedural sobre un búfer RGBA supermuestreado; sin biblioteca de imágenes.
//!
//! **Alcance de esta primera versión** (`docs/open-questions.md` J.19): el color se recalcula en
//! los puntos de sincronización existentes —arranque, `refresh_now`, comandos de alerta,
//! pausar/reanudar— y no en un ciclo real cada 30 s: el planificador en segundo plano (T020), la
//! emisión de eventos (T021) y la reanudación automática al arrancar (T022) siguen sin
//! implementar. El botón de cierre (`X`) minimiza siempre a la bandeja; la pregunta "minimizar o
//! salir" con opción de recordar (FR-012) queda pendiente de un diálogo propio.

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{image::Image, AppHandle, Manager};

use crate::domain::salud::tray_state;
use crate::domain::tipos::HealthState;
use crate::persistence::db::AppState;
use crate::platform::rotulos::{locale_actual, t, tp};
use crate::platform::ventana;

/// Lado del icono final, en píxeles. Windows lo reescala a 16/20/24 según el DPI.
const LADO: u32 = 32;
/// Se dibuja a `LADO * SUPERMUESTREO` y se reduce por promedio: da bordes suaves a las elipses y a
/// las diagonales sin un rasterizador con antialias.
const SUPERMUESTREO: u32 = 4;

const ID_ABRIR: &str = "tray-abrir";
const ID_RESUMEN: &str = "tray-resumen";
const ID_PAUSAR: &str = "tray-pausar";
const ID_SALIR: &str = "tray-salir";

/// El `TrayIcon` ya construido, gestionado por Tauri para poder actualizarlo desde cualquier
/// comando sin reconstruirlo entero cada vez.
struct BandejaIcono(TrayIcon);

fn color_rgba(estado: HealthState) -> [u8; 4] {
    // Mismos valores que `--sdm-{ok,warn,crit,unknown}` en tema claro (`tokens.css`): el icono no
    // inventa un color que la interfaz no use ya.
    match estado {
        HealthState::Ok => [0x2f, 0x72, 0x56, 0xff],
        HealthState::Warn => [0x7d, 0x56, 0x19, 0xff],
        HealthState::Crit => [0xa8, 0x3d, 0x45, 0xff],
        HealthState::Unknown => [0x5d, 0x61, 0x6d, 0xff],
    }
}

/// Glifo del icono según el estado agregado. La forma cambia con el estado, no solo el color: así
/// se distingue a 16 px y sin depender del color (constitución §VII).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum GlifoBandeja {
    /// Disco lleno. Todo en orden.
    Correcto,
    /// Disco con un «!». Alguna advertencia.
    Advertencia,
    /// Disco con una «×». Algo crítico.
    Critico,
    /// Disco hueco (solo contorno). Sin datos, sin discos monitorizados o fallo de recopilador.
    SinDatos,
    /// Dos barras de pausa. Monitorización en pausa y sin crítico vigente.
    Pausa,
}

/// Traduce el estado agregado (ya resuelto por `tray_state`, regla B.5) y la pausa al glifo.
fn glifo(estado: HealthState, pausado: bool) -> GlifoBandeja {
    // El crítico se comprueba primero: `tray_state` conserva el rojo aunque la monitorización esté
    // en pausa (B.5, regla 1), y el icono tiene que decir lo mismo.
    match estado {
        HealthState::Crit => GlifoBandeja::Critico,
        HealthState::Warn => GlifoBandeja::Advertencia,
        HealthState::Ok => GlifoBandeja::Correcto,
        HealthState::Unknown if pausado => GlifoBandeja::Pausa,
        HealthState::Unknown => GlifoBandeja::SinDatos,
    }
}

/// El color de estado bajo el glifo. `Pausa` y `SinDatos` comparten el gris de `Unknown`.
fn color_de(glifo: GlifoBandeja) -> [u8; 4] {
    match glifo {
        GlifoBandeja::Correcto => color_rgba(HealthState::Ok),
        GlifoBandeja::Advertencia => color_rgba(HealthState::Warn),
        GlifoBandeja::Critico => color_rgba(HealthState::Crit),
        GlifoBandeja::SinDatos | GlifoBandeja::Pausa => color_rgba(HealthState::Unknown),
    }
}

/// Lienzo RGBA opaco/transparente sobre el que se pinta a mano. Las formas son de borde duro; el
/// suavizado lo da la reducción posterior (`reducir`).
struct Lienzo {
    lado: i32,
    pix: Vec<[u8; 4]>,
}

impl Lienzo {
    fn nuevo(lado: u32) -> Self {
        Self {
            lado: lado as i32,
            pix: vec![[0, 0, 0, 0]; (lado * lado) as usize],
        }
    }

    fn poner(&mut self, x: i32, y: i32, color: [u8; 4]) {
        if (0..self.lado).contains(&x) && (0..self.lado).contains(&y) {
            self.pix[(y * self.lado + x) as usize] = color;
        }
    }

    /// Rectángulo relleno `[x0, x1) × [y0, y1)`.
    fn rect(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: [u8; 4]) {
        for y in y0..y1 {
            for x in x0..x1 {
                self.poner(x, y, color);
            }
        }
    }

    /// Rectángulo con las esquinas redondeadas a `radio`.
    fn redondeado(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, radio: i32, color: [u8; 4]) {
        for y in y0..y1 {
            for x in x0..x1 {
                let ax = x.clamp(x0 + radio, x1 - radio - 1);
                let ay = y.clamp(y0 + radio, y1 - radio - 1);
                let (dx, dy) = (x - ax, y - ay);
                if dx * dx + dy * dy <= radio * radio {
                    self.poner(x, y, color);
                }
            }
        }
    }

    /// Elipse rellena centrada en `(cx, cy)` con semiejes `rx`, `ry`.
    fn elipse(&mut self, cx: i32, cy: i32, rx: i32, ry: i32, color: [u8; 4]) {
        for y in (cy - ry)..=(cy + ry) {
            for x in (cx - rx)..=(cx + rx) {
                let a = (x - cx) as f32 / rx as f32;
                let b = (y - cy) as f32 / ry as f32;
                if a * a + b * b <= 1.0 {
                    self.poner(x, y, color);
                }
            }
        }
    }

    /// Contorno de una elipse: dentro de `(rx, ry)` y fuera de `(rx - g, ry - g)`.
    fn anillo(&mut self, cx: i32, cy: i32, rx: i32, ry: i32, g: i32, color: [u8; 4]) {
        let dentro = |x: i32, y: i32, ex: f32, ey: f32| {
            let a = (x - cx) as f32 / ex;
            let b = (y - cy) as f32 / ey;
            a * a + b * b <= 1.0
        };
        for y in (cy - ry)..=(cy + ry) {
            for x in (cx - rx)..=(cx + rx) {
                if dentro(x, y, rx as f32, ry as f32)
                    && !dentro(x, y, (rx - g).max(1) as f32, (ry - g).max(1) as f32)
                {
                    self.poner(x, y, color);
                }
            }
        }
    }

    /// Segmento grueso de `(ax, ay)` a `(bx, by)`, grosor `g`.
    fn diagonal(&mut self, ax: i32, ay: i32, bx: i32, by: i32, g: i32, color: [u8; 4]) {
        let (fx0, fy0) = (ax.min(bx) - g, ay.min(by) - g);
        let (fx1, fy1) = (ax.max(bx) + g, ay.max(by) + g);
        let (ax, ay, bx, by) = (ax as f32, ay as f32, bx as f32, by as f32);
        let largo2 = (bx - ax).powi(2) + (by - ay).powi(2);
        for y in fy0..=fy1 {
            for x in fx0..=fx1 {
                let t = if largo2 == 0.0 {
                    0.0
                } else {
                    (((x as f32 - ax) * (bx - ax) + (y as f32 - ay) * (by - ay)) / largo2)
                        .clamp(0.0, 1.0)
                };
                let (px, py) = (ax + t * (bx - ax), ay + t * (by - ay));
                if (x as f32 - px).powi(2) + (y as f32 - py).powi(2) <= (g as f32 / 2.0).powi(2) {
                    self.poner(x, y, color);
                }
            }
        }
    }
}

/// La silueta del disco: elipse superior marcada + cuerpo + elipse inferior (un cilindro de datos,
/// la misma metáfora que el icono `i-diskStack` del catálogo). `y0`/`y1` son los centros de las dos
/// tapas.
fn disco(l: &mut Lienzo, cx: i32, rx: i32, ry: i32, y0: i32, y1: i32, color: [u8; 4]) {
    l.elipse(cx, y0, rx, ry, color);
    l.rect(cx - rx, y0, cx + rx + 1, y1, color);
    l.elipse(cx, y1, rx, ry, color);
}

/// Reduce el lienzo grande a `LADO × LADO` promediando cada bloque `s × s` con alfa premultiplicado
/// (así el borde contra la parte transparente no se ensucia de negro).
fn reducir(grande: &Lienzo, s: u32) -> Vec<u8> {
    let lado = grande.lado as u32 / s;
    let mut salida = Vec::with_capacity((lado * lado * 4) as usize);
    for y in 0..lado {
        for x in 0..lado {
            let (mut ar, mut ag, mut ab, mut aa) = (0u32, 0u32, 0u32, 0u32);
            for dy in 0..s {
                for dx in 0..s {
                    let px =
                        grande.pix[(((y * s + dy) * grande.lado as u32) + (x * s + dx)) as usize];
                    let alfa = px[3] as u32;
                    ar += px[0] as u32 * alfa;
                    ag += px[1] as u32 * alfa;
                    ab += px[2] as u32 * alfa;
                    aa += alfa;
                }
            }
            if aa == 0 {
                salida.extend_from_slice(&[0, 0, 0, 0]);
            } else {
                let n = s * s;
                salida.extend_from_slice(&[
                    (ar / aa) as u8,
                    (ag / aa) as u8,
                    (ab / aa) as u8,
                    (aa / n) as u8,
                ]);
            }
        }
    }
    salida
}

/// El búfer RGBA del icono (`LADO × LADO`). Separado de `icono` para poder afirmar sobre él sin
/// pasar por `Image`.
fn pixeles(glifo: GlifoBandeja) -> Vec<u8> {
    let s = SUPERMUESTREO as i32;
    let color = color_de(glifo);
    let blanco = [0xff, 0xff, 0xff, 0xff];
    let cx = 16 * s;
    let mut l = Lienzo::nuevo(LADO * SUPERMUESTREO);

    // Tile del color de estado, con un margen para que no toque los bordes del área del icono.
    l.redondeado(2 * s, 2 * s, 30 * s, 30 * s, 6 * s, color);

    // El cilindro de datos: tapas elípticas anchas (radios 9 × 3) centradas en y=9 y y=23, la misma
    // metáfora que `i-diskStack`.
    let (rx, ry) = (9 * s, 3 * s);
    let (y0, y1) = (9 * s, 23 * s);
    let cilindro = |l: &mut Lienzo, c: [u8; 4]| disco(l, cx, rx, ry, y0, y1, c);

    match glifo {
        GlifoBandeja::Pausa => {
            l.redondeado(10 * s, 9 * s, 14 * s, 23 * s, s, blanco);
            l.redondeado(18 * s, 9 * s, 22 * s, 23 * s, s, blanco);
        }
        GlifoBandeja::SinDatos => {
            // Solo el contorno: un disco «vacío».
            cilindro(&mut l, blanco);
            disco(
                &mut l,
                cx,
                rx - 2 * s,
                (ry - s).max(s),
                y0 + s,
                y1 - s,
                color,
            );
            l.anillo(cx, y0, rx, ry, s, blanco);
        }
        GlifoBandeja::Correcto => {
            cilindro(&mut l, blanco);
            l.anillo(cx, y0, rx, ry, s, color); // tapa
            l.anillo(cx, 16 * s, rx, ry, s, color); // costura del apilado
        }
        GlifoBandeja::Advertencia => {
            cilindro(&mut l, blanco);
            l.anillo(cx, y0, rx, ry, s, color);
            l.rect(cx - 2 * s, 11 * s, cx + 2 * s, 19 * s, color);
            l.rect(cx - 2 * s, 21 * s, cx + 2 * s, 25 * s, color);
        }
        GlifoBandeja::Critico => {
            cilindro(&mut l, blanco);
            l.anillo(cx, y0, rx, ry, s, color);
            l.diagonal(12 * s, 12 * s, 20 * s, 20 * s, 3 * s, color);
            l.diagonal(20 * s, 12 * s, 12 * s, 20 * s, 3 * s, color);
        }
    }

    reducir(&l, SUPERMUESTREO)
}

fn icono(glifo: GlifoBandeja) -> Image<'static> {
    Image::new_owned(pixeles(glifo), LADO, LADO)
}

/// El texto de resumen: mismas claves y misma lógica que `globalLabel` en `+layout.svelte`, para
/// que el menú de la bandeja diga exactamente lo mismo que la barra de la ventana principal.
fn texto_resumen(locale: &str, pausado: bool, estados: &[HealthState]) -> String {
    if pausado {
        return t(locale, "global.paused");
    }
    if estados.is_empty() {
        return t(locale, "global.noDevices");
    }
    let necesitan_atencion = estados
        .iter()
        .filter(|e| matches!(e, HealthState::Warn | HealthState::Crit))
        .count();
    if necesitan_atencion == 0 {
        return t(locale, "global.allGood");
    }
    tp(locale, "global.needsAttention", necesitan_atencion as i64)
}

fn construir_menu(
    app: &AppHandle,
    pausado: bool,
    resumen: &str,
) -> tauri::Result<Menu<tauri::Wry>> {
    let locale = locale_actual();
    let abrir = MenuItem::with_id(app, ID_ABRIR, t(locale, "tray.open"), true, None::<&str>)?;
    let resumen_item = MenuItem::with_id(app, ID_RESUMEN, resumen, false, None::<&str>)?;
    let pausar = MenuItem::with_id(
        app,
        ID_PAUSAR,
        if pausado {
            t(locale, "nav.resume")
        } else {
            t(locale, "nav.pause")
        },
        true,
        None::<&str>,
    )?;
    let salir = MenuItem::with_id(app, ID_SALIR, t(locale, "tray.exit"), true, None::<&str>)?;
    Menu::with_items(
        app,
        &[
            &abrir,
            &resumen_item,
            &PredefinedMenuItem::separator(app)?,
            &pausar,
            &PredefinedMenuItem::separator(app)?,
            &salir,
        ],
    )
}

/// Recalcula color, menú y texto emergente a partir del estado real y los aplica al icono ya
/// construido. Se llama tras cada acción que puede cambiar el color: arranque, `refresh_now`,
/// las seis acciones sobre alertas y pausar/reanudar (`docs/open-questions.md` J.19).
pub fn actualizar(app: &AppHandle) {
    let Some(bandeja) = app.try_state::<BandejaIcono>() else {
        return;
    };
    let estado = app.state::<AppState>();
    let conn = estado
        .conn
        .lock()
        .expect("el mutex de la conexión no se envenena: sin pánicos dentro");
    let respuesta = match crate::commands::get_devices_impl(&conn) {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(error = ?e, "no se pudo recalcular el icono de la bandeja");
            return;
        }
    };
    drop(conn);
    let pausado = estado
        .paused
        .lock()
        .expect("el mutex de pausa no se envenena: sin pánicos dentro")
        .is_some();

    let estados: Vec<HealthState> = respuesta.devices.iter().map(|d| d.state).collect();
    let fallo_recopilador = respuesta.sources.iter().any(|s| {
        matches!(
            s.status,
            crate::commands::SourceStatus::Timeout | crate::commands::SourceStatus::Error
        )
    });
    let estado_agregado = tray_state(pausado, fallo_recopilador, &estados);
    let locale = locale_actual();
    let resumen = texto_resumen(locale, pausado, &estados);
    // El texto emergente nombra la aplicación: entre muchos iconos de bandeja, «Todo en orden» a
    // secas no dice a quién pertenece. El formato vive en el diccionario (`tray.tooltip`).
    let tooltip = t(locale, "tray.tooltip").replace("{summary}", &resumen);

    if let Err(e) = bandeja
        .0
        .set_icon(Some(icono(glifo(estado_agregado, pausado))))
    {
        tracing::warn!(error = %e, "no se pudo actualizar el icono de la bandeja");
    }
    if let Err(e) = bandeja.0.set_tooltip(Some(&tooltip)) {
        tracing::warn!(error = %e, "no se pudo actualizar el texto emergente de la bandeja");
    }
    match construir_menu(app, pausado, &resumen) {
        Ok(menu) => {
            if let Err(e) = bandeja.0.set_menu(Some(menu)) {
                tracing::warn!(error = %e, "no se pudo actualizar el menú de la bandeja");
            }
        }
        Err(e) => tracing::warn!(error = %e, "no se pudo construir el menú de la bandeja"),
    }
}

fn alternar_pausa(app: &AppHandle) {
    let estado = app.state::<AppState>();
    let ya_pausado = {
        let mut pausa = estado
            .paused
            .lock()
            .expect("el mutex de pausa no se envenena: sin pánicos dentro");
        let estaba = pausa.is_some();
        *pausa = if estaba {
            None
        } else {
            Some(
                time::OffsetDateTime::now_utc()
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap_or_default(),
            )
        };
        estaba
    };
    tracing::info!(
        pausado_ahora = !ya_pausado,
        "monitorización conmutada desde la bandeja"
    );
    actualizar(app);
}

/// Construye el icono inicial y registra sus manejadores de clic. Se llama una sola vez, desde
/// `setup()`. El color se calcula de inmediato con `actualizar()`, no se deja en un valor inicial
/// inventado.
pub fn instalar(app: &AppHandle) -> tauri::Result<()> {
    let menu_inicial = construir_menu(app, false, "")?;

    let tray = TrayIconBuilder::new()
        .icon(icono(GlifoBandeja::SinDatos))
        .menu(&menu_inicial)
        .tooltip(t(locale_actual(), "app.name"))
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            ID_ABRIR => ventana::restaurar_ventana_principal(app),
            ID_PAUSAR => alternar_pausa(app),
            ID_SALIR => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                ventana::restaurar_ventana_principal(tray.app_handle());
            }
        })
        .build(app)?;

    app.manage(BandejaIcono(tray));
    actualizar(app);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn el_resumen_pausado_ignora_los_dispositivos() {
        assert_eq!(texto_resumen("es", true, &[HealthState::Crit]), "En pausa");
    }

    #[test]
    fn el_resumen_sin_dispositivos_lo_dice() {
        assert_eq!(texto_resumen("es", false, &[]), "Sin discos monitorizados");
    }

    #[test]
    fn el_resumen_todo_correcto_lo_dice() {
        assert_eq!(
            texto_resumen("es", false, &[HealthState::Ok, HealthState::Ok]),
            "Todo en orden"
        );
    }

    #[test]
    fn el_resumen_cuenta_los_que_necesitan_atencion() {
        assert_eq!(
            texto_resumen(
                "es",
                false,
                &[HealthState::Ok, HealthState::Warn, HealthState::Crit]
            ),
            "2 discos necesitan atención"
        );
    }

    #[test]
    fn el_resumen_en_singular_no_usa_plural() {
        assert_eq!(
            texto_resumen("es", false, &[HealthState::Warn]),
            "1 disco necesita atención"
        );
    }

    #[test]
    fn el_tooltip_nombra_la_aplicacion() {
        let tooltip = t("es", "tray.tooltip").replace("{summary}", "Todo en orden");
        assert!(
            tooltip.starts_with("SmartDisk Monitor"),
            "el tooltip no nombra la aplicación: {tooltip:?}"
        );
        assert!(tooltip.contains("Todo en orden"));
    }

    #[test]
    fn el_glifo_critico_gana_a_la_pausa() {
        // B.5 regla 1: el rojo (y su «×») se mantienen aunque la monitorización esté en pausa.
        assert_eq!(glifo(HealthState::Crit, true), GlifoBandeja::Critico);
    }

    #[test]
    fn el_glifo_de_pausa_solo_sin_critico() {
        assert_eq!(glifo(HealthState::Unknown, true), GlifoBandeja::Pausa);
        assert_eq!(glifo(HealthState::Unknown, false), GlifoBandeja::SinDatos);
    }

    #[test]
    fn el_glifo_mapea_los_estados_de_color() {
        assert_eq!(glifo(HealthState::Ok, false), GlifoBandeja::Correcto);
        assert_eq!(glifo(HealthState::Warn, false), GlifoBandeja::Advertencia);
    }

    #[test]
    fn el_icono_tiene_el_tamano_correcto_y_pinta_tile_y_glifo() {
        let px = pixeles(GlifoBandeja::Correcto);
        assert_eq!(px.len(), (LADO * LADO * 4) as usize);

        let en = |x: u32, y: u32| {
            let i = ((y * LADO + x) * 4) as usize;
            [px[i], px[i + 1], px[i + 2], px[i + 3]]
        };
        // Esquina: fuera del tile redondeado, transparente.
        assert_eq!(en(0, 0)[3], 0, "la esquina debería ser transparente");
        // Centro: el disco blanco.
        let c = en(LADO / 2, LADO / 2);
        assert!(
            c[0] > 200 && c[1] > 200 && c[2] > 200 && c[3] == 255,
            "el centro debería ser el disco blanco: {c:?}"
        );
        // Bajo el tile, sobre el borde: el verde de «ok» (no blanco, no transparente).
        let borde = en(LADO / 2, 4);
        assert!(
            borde[3] == 255 && borde[1] > borde[0] && borde[1] > borde[2],
            "el borde superior del tile debería ser verdoso: {borde:?}"
        );
    }

    #[test]
    fn cada_glifo_produce_un_dibujo_distinto() {
        let todos = [
            pixeles(GlifoBandeja::Correcto),
            pixeles(GlifoBandeja::Advertencia),
            pixeles(GlifoBandeja::Critico),
            pixeles(GlifoBandeja::SinDatos),
            pixeles(GlifoBandeja::Pausa),
        ];
        for (i, a) in todos.iter().enumerate() {
            for b in todos.iter().skip(i + 1) {
                assert_ne!(a, b, "dos glifos comparten dibujo");
            }
        }
    }

    /// Ayuda de QA (no es una prueba de regresión): vuelca los cinco iconos como BMP de 32×32 a
    /// `src-tauri/target/bandeja/` para revisarlos a ojo.
    /// `cargo test volcar_iconos_bmp -- --ignored`
    #[test]
    #[ignore = "ayuda visual manual, no una comprobación automática"]
    fn volcar_iconos_bmp() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/bandeja");
        std::fs::create_dir_all(&dir).expect("crear target/bandeja");
        for (nombre, glifo) in [
            ("correcto", GlifoBandeja::Correcto),
            ("advertencia", GlifoBandeja::Advertencia),
            ("critico", GlifoBandeja::Critico),
            ("sin-datos", GlifoBandeja::SinDatos),
            ("pausa", GlifoBandeja::Pausa),
        ] {
            let bmp = a_bmp(&pixeles(glifo), LADO);
            std::fs::write(dir.join(format!("{nombre}.bmp")), bmp).expect("escribir BMP");
        }
        eprintln!("iconos escritos en {}", dir.display());
    }

    /// RGBA → BMP de 32 bits, compuesto sobre un gris claro para que se vea el margen transparente.
    fn a_bmp(rgba: &[u8], lado: u32) -> Vec<u8> {
        let fila = lado * 4;
        let datos = lado * fila;
        let mut b = Vec::with_capacity(54 + datos as usize);
        b.extend_from_slice(b"BM");
        b.extend_from_slice(&(54 + datos).to_le_bytes());
        b.extend_from_slice(&0u32.to_le_bytes());
        b.extend_from_slice(&54u32.to_le_bytes());
        b.extend_from_slice(&40u32.to_le_bytes());
        b.extend_from_slice(&(lado as i32).to_le_bytes());
        b.extend_from_slice(&(-(lado as i32)).to_le_bytes()); // negativo: filas de arriba a abajo
        b.extend_from_slice(&1u16.to_le_bytes());
        b.extend_from_slice(&32u16.to_le_bytes());
        b.extend_from_slice(&0u32.to_le_bytes());
        b.extend_from_slice(&datos.to_le_bytes());
        b.extend_from_slice(&[0u8; 16]);
        for px in rgba.chunks_exact(4) {
            let a = px[3] as u32;
            let mezcla = |c: u8| ((c as u32 * a + 0xdd * (255 - a)) / 255) as u8;
            b.extend_from_slice(&[mezcla(px[2]), mezcla(px[1]), mezcla(px[0]), 0xff]);
        }
        b
    }
}
