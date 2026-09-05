# Fase 1 — Guía de validación

**Funcionalidad**: SmartDisk Monitor 1.0 · **Fecha**: 2026-09-04 · **Plan**: [plan.md](plan.md)

Cómo comprobar que lo construido funciona de verdad. No es documentación de puesta en marcha —eso
está en `README.md`—: es el guion de validación de esta funcionalidad, eslabón a eslabón.

## Requisitos previos

- Windows 10 (1809+), 11 o Server 2016–2025, x64, con Experiencia de escritorio y WebView2 Runtime.
- Node 24 (mínimo 20), pnpm 11.6.0, Rust 1.94.0. Las versiones exactas están fijadas en `.nvmrc`,
  `package.json` y `rust-toolchain.toml`.
- **Terminal elevada** para todo lo que toque discos: sin privilegios de administrador no hay lectura
  de salud, y la aplicación **no arranca a medias** (US-001, FR-032).
- Para validar los eslabones 2 y 3 hace falta hardware real y variado: al menos un NVMe o SATA
  interno y un dispositivo USB. Un dispositivo del que **no** se puedan leer datos de salud no es un
  estorbo, es un caso de prueba imprescindible.

## Puertas que se pasan siempre

Antes de dar por bueno cualquier eslabón. El orden importa: un error de tipos invalida todo lo que
venga después (constitución §XIII).

```powershell
pnpm check          # cero errores y cero avisos, sin excepción
pnpm lint           # prettier + eslint
pnpm verify         # recursos, copia única del sistema de diseño, i18n, fronteras
pnpm test           # lógica en Node
pnpm test:component # componentes en Chromium
pnpm test:e2e       # interfaz con IPC propio
pnpm test:a11y      # axe en las seis pantallas, ambos temas
cargo test          # desde src-tauri/
cargo clippy --all-targets -- -D warnings
pnpm docs:check     # el consolidado no va por detrás
```

Un aviso de accesibilidad **no es una molestia**: es un incumplimiento del principio VII. Si uno no
se puede resolver, se registra en `docs/known-issues.md` y el silencio del código enlaza a su entrada
por número.

## Validación por eslabón

Cada bloque comprueba lo que el eslabón promete, **y también lo que no debe pasar**. La segunda parte
es la que importa: este producto falla de forma silenciosa o no falla.

### 1 · Persistencia

```powershell
pnpm app:dev        # muestra UAC: la aplicación va elevada
```

- La base se crea en `%ProgramData%\SmartDisk Monitor\` con WAL activo.
- **Antes de escribir código de retención**: comprobar que los umbrales de espacio libre están
  registrados en `docs/open-questions.md` como `PROPUESTO` (riesgo R4). Si no lo están, el trabajo se
  para: programar una decisión no escrita es la infracción de proceso más grave del proyecto.
- Cerrar el proceso de golpe durante una escritura y volver a abrir: el historial queda íntegro y la
  aplicación no arranca en un estado a medias (FR-035).
- Llenar el volumen hasta cruzar el umbral de parada: **la vigilancia sigue**, la escritura se
  detiene, y la interfaz lo dice (FR-020a/b, SC-015a).
- Comprobar que **nada se ha borrado solo** para hacer sitio (FR-020c).

### 2 · Inventario

- Todos los discos aparecen, con los compatibles seleccionados y sin haber configurado nada (SC-001).
- Conectar un disco con la aplicación abierta: aparece en menos de 60 segundos, sin reiniciar y sin
  que reaparezca el asistente (SC-005).
- Retirar **con** expulsión segura: apunte de inventario, sin alerta. Retirar **sin** aviso: alerta
  crítica, o advertencia si es extraíble.
- Volver a conectarlo: continúa su historial. Si la identidad se dedujo por huella, la interfaz lo
  marca como inferida.
- **Aquí se activa la puerta de DTO generados**: comprobar que `src/lib/api/types.ts` ya no se
  mantiene a mano.

### 3 · Salud

- Cada métrica muestra su procedencia y cuándo se leyó; si el dato está caduco, lo dice (FR-005).
- Un dispositivo sin datos de salud aparece **en gris**, como «Sin datos SMART», sin alerta y sin
  rojo (SC-003). Es el caso que más veces se rompe al refactorizar.
- Todo dato ausente dice «No disponible». Ni un cero (SC-002).
- Un disco monitorizado sin lecturas frescas queda en **desconocido**, no en correcto.
- Medir el tiempo de la cascada de modos de acceso en un dispositivo USB o RAID (riesgo R2): declarar
  «no compatible» es un resultado correcto; tardar tanto que degrade el ciclo, no.

### 4 · Métricas y series

- El eje temporal cubre el intervalo pedido **completo**, aunque falten datos en los extremos.
- Suspender el equipo y despertarlo: el hueco se dibuja como hueco, sin interpolar.
- Si se muestran promedios, el pie declara la resolución (FR-016).

### 5 · Alertas

- Una condición que se repite N veces produce **un** grupo con contador N (SC-004).
- Reconocer: sale de pendientes, queda distinguida, **el color del disco no cambia** (SC-006).
- Silenciar 15 min: deja de notificar, el color sigue igual.
- Resolver la condición: el grupo pasa a resuelto y deja de contar.
- El icono de la bandeja recorre los cuatro estados: verde, ámbar, rojo y gris.
- **Medir R1**: si las notificaciones del sistema no llegan desde el proceso elevado, activar la
  alternativa ya decidida (ventana propia con `Toast`).

### 6 · Eventos

- Cerrar y reabrir: los eventos ya registrados **no se duplican**.
- Los de atribución deducida van etiquetados como asociación inferida.
- El texto original se muestra como texto plano, nunca interpretado.
- **Medir R3** con la lista virtualizada: 20 discos y 5.000 eventos, comprobando que la interfaz
  nunca deja de responder más de 50 ms seguidos (SC-007, SC-009).

### 7 · Pruebas

- Cada prueba pide confirmación con acción, destino, impacto y orden literal (SC-012).
- Con la prueba en curso, la interfaz sigue respondiendo y se puede cancelar.
- Forzar el límite de temperatura: la prueba se detiene sola y explica por qué.
- En un dispositivo sin autodiagnóstico, la opción está deshabilitada **con el motivo escrito**
  (FR-025), no simplemente apagada.

### 8 · Informes y diagnóstico

- Exportar en los tres formatos: el contenido corresponde al intervalo y los discos pedidos.
- **Abrir el paquete de diagnóstico y leerlo**: sin números de serie, sin nombre de equipo, sin rutas
  de usuario (SC-013). Esta comprobación se hace a mano, mirando dentro; no basta con que el código
  diga que anonimiza.
- El registro de actividad va dentro, sujeto a la misma anonimización (FR-029c).

### 9 · Ajustes y ciclo de vida

- Un valor fuera de límites se rechaza con explicación, y el anterior se mantiene.
- Cambiar idioma y tema: efecto inmediato y sobrevive al reinicio.
- Pasar a batería: se espacian las métricas rápidas, **no** las que alimentan alertas graves.
- Pausar y reiniciar la aplicación: **siempre reanuda** (FR-031).
- Activar el modo detallado y abrir la carpeta del registro (FR-029a/b).
- Borrar todos los datos: se confirma explícitamente y se dice qué se borró.

### 10 · Empaquetado

```powershell
pnpm app:build
```

- Instalar en un equipo **sin conexión de red**: funciona sin descargar nada (SC-014).
- Verificar con un monitor de red que la aplicación no hace **ninguna** petición saliente.
- Abrir dos veces: la segunda trae al frente la primera (FR-033).
- Desinstalar: historial y configuración **se conservan**, y se explica cómo eliminarlos.
- Migraciones verificadas desde cada versión publicada anterior, no solo desde cero.

## Revisión por pantalla

Además de lo anterior, toda pantalla pasa la definición de terminado de `docs/ui-design.md` §8 antes
de darse por cerrada. Es revisión humana y no la sustituye ninguna herramienta:

tema claro y oscuro · acento del sistema y azul de respaldo · 1024 × 560 y 1280 × 720 · escalado
125 %, 150 % y 200 % · estados cargando, vacío, no compatible, error de fuente y dato obsoleto ·
teclado y foco · textos en los dos idiomas, `aria-label` incluidos.

Dos comprobaciones que se olvidan siempre y que el proyecto ya ha pagado una vez:

- **Un valor ausente en cada métrica**: «No disponible» tiene que caber y distinguirse de una cifra.
- **Un acento del sistema claro** (por ejemplo el amarillo `#ffb900`) en **ambos** temas: el texto
  sobre el acento y el de acento sobre el material siguen cumpliendo AA.
