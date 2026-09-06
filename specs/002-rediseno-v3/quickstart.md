# Quickstart — validación del rediseño v3

Guía para comprobar que cada PR entrega lo que dice. No es la lista de tareas (eso es `tasks.md`); es cómo se valida.

## Prerrequisitos

```
pnpm install
pnpm app:dev          # arranca con UAC (app elevada)
```

Comandos de verificación que **todo PR** debe dejar en verde antes de proponerse:

```
pnpm verify           # assets, tokens, i18n, boundaries
pnpm check            # tipos + accesibilidad, cero avisos
pnpm test             # lógica (Node)
pnpm test:component    # componentes (Chromium)
pnpm lint
```

Y para los PR que tocan pantallas o el asistente:

```
pnpm test:e2e:smoke
pnpm test:a11y
```

PR 5 además:

```
cd src-tauri && cargo test && cargo clippy --all-targets -- -D warnings
```

## Escenarios por historia

### US1 — tokens v3 (PR 1)

1. Abrir la app en tema claro y oscuro. El acento es morado, no azul; el rojo crítico es bermellón.
2. En oscuro, inspeccionar un botón primario: su texto usa `--sdm-on-accent` (tinta), contraste ≥ 4,5:1.
3. `pnpm verify:tokens` en verde (una copia, cero literales).
4. Comparar cada pantalla con `design/propuesta-redisenov2/mockups/smartdisk-v3.html` en los dos temas.
5. `docs/decisions.md` contiene ADR-034 y ADR-035; `docs/ui-design.md` describe v3; `docs/open-questions.md` lista los ratios medidos.

### US2 — iconos (PR 2)

1. Abrir `design/propuesta-redisenov2/mockups/icons-hoja-de-contacto.html`: los 15 iconos a 4 tamaños en los 2 temas.
2. En la app, un `<Icon>` sin `label` no lo anuncia el lector de pantalla; con `label` lo anuncia como imagen con nombre.
3. `$lib/design/icons.ts` exporta `healthIcon`, `eventLevelIcon`, `busIcon`, `testIcon`; prueba unitaria de `busIcon("USB Mass Storage") === "usb"`.

### US3 — chrome (PR 3)

1. A 1024 × 560: la barra lateral ocupa 74 px; el contenido, 950.
2. Navegar Panel → Detalle de disco → Alertas: el título de la barra de herramientas cambia con la ruta (ya no dice siempre «Panel general»).
3. Con un disco monitorizado en `ok` y ninguno en `warn`/`crit`: la píldora dice «Todo en orden», nunca «Sin discos monitorizados», y coincide con el icono del pie del riel.
4. Teclado: Tab recorre las seis secciones + Acerca de; cada una anuncia su nombre; la de Alertas con avisos anuncia «Alertas, N sin revisar».
5. La lista de discos ya no está en la barra lateral; «Pausar recopilación» está en Ajustes y en el menú de la bandeja.

### US4 — gráfica (PR 4)

1. Detalle de un disco con serie que tiene un hueco: se ve el trazo, el eje Y con 4 marcas, el relleno bajo la curva, y una banda gris sobre el hueco con «sin datos HH:MM – HH:MM».
2. Redimensionar la ventana: el grosor del trazo no cambia ni desaparece.
3. Serie sin muestras en el intervalo: marco + eje + «Sin muestras en las últimas 24 h», sin línea a cero.
4. Prueba de componente de `Sparkline` con `points` de dos tramos → dos `<polyline>`.

### US10 — perfiles de alerta (PR 5) — **modo plan**

1. Ajustes → Alertas → elegir «Prudente»: los doce umbrales se escriben con los valores de `data-model.md` §3.3 y `settings.alerts.profile === "cautious"`.
2. Cambiar `wearWarnPercent` a mano → `profile === "custom"`, la UI muestra «Personalizado (a partir de Prudente)».
3. Guardar `wearCritPercent` menor que `wearWarnPercent` → `AppError`, el control revierte, el error aparece junto al control.
4. `cargo test`: cada regla parametrizada nueva tiene sus 5 pruebas (activación, no-activación ante dato ausente, histéresis, deduplicación, ciclo de recaída).
5. Disco que declara límite térmico de 70 °C con perfil «Solo lo grave»: avisa a 70 °C por el fabricante, no por el perfil; `temp_above_vendor_limit` sigue activa.
6. `docs/alert-rules.md`, `docs/ui-contract.md`, `docs/data-model.md` actualizados; ADR de motor parametrizado en `docs/decisions.md`.

### US5 / US6 — panel y detalle (PR 6)

1. Panel con 4 discos, uno en advertencia térmica: el `HeroPanel` muestra ese disco con curva de fondo, cifra a 76 px, umbral discontinuo y dos acciones.
2. Panel con todos en `ok`: el héroe muestra el disco de sistema, «Todo en orden», cifra en `--sdm-text` (no verde), solo «Abrir el disco».
3. El panel se pinta de inmediato; las sparklines aparecen tras el primer render (carga perezosa). Con 20 discos, `performance` no muestra tareas > 50 ms al desplazarse (SC-006). Medición anotada en `docs/open-questions.md`.
4. Disco USB sin SMART en la rejilla: en gris, magnitudes como «—», barra de capacidad presente, nunca en el héroe.
5. «Reparto de estados» sustituye al anillo; `HealthDonut` sigue en el catálogo y en su prueba.
6. Detalle: 4 `MetricCard` con icono + sparkline + procedencia; el `SegmentedControl` de intervalo está junto a la gráfica; un valor `null` se muestra como «No disponible» a `text-lg`, no como cifra; ninguna cifra usa peso 800.

### US7 — pantallas secundarias (PR 7)

1. Pruebas con una prueba en curso: bloque arriba, `ProgressBar emphasis="display"` (12 px, degradado), cifra de progreso a 58 px.
2. Sin prueba en curso: el bloque no aparece, las tarjetas suben, sin hueco.
3. Eventos: nivel `error` como cuadrado de 26 px con `#i-bolt` y `aria-label`; altura de fila sigue en 42 px.
4. Alertas: píldora de severidad `crit` con `#i-bolt` y `×N` en `.sdm-num`.
5. Ajustes: «Borrado de datos» separada con `space-8` y borde crítico; el fondo no está teñido.

### US9 — acento opcional (PR 8)

1. Instalación nueva: `useSystemAccent === false`; el acento es Ciruela.
2. Activar el interruptor con un acento de Windows amarillo de prueba: el botón primario y el texto de acento cumplen AA en los dos temas; cambiar de tema recalcula `--sdm-accent-fg`.
3. Desactivar: vuelve el morado. Reiniciar: la preferencia persiste.

### US8 — asistente inicial (PR 9)

1. Con `settings.onboarding.completedAt` nulo, arrancar: redirige a `/onboarding`, sin riel ni barra de herramientas.
2. Recorrer los cuatro pasos; «Omitir y usar los valores de fábrica» visible en todos.
3. Paso 2: excluir un disco, ponerle alias; el USB se explica como «no es una avería», sin rojo.
4. Paso 3: elegir un perfil, ver la tabla de umbrales reaccionar; pulsar «Omitir» → se aplica Equilibrado, se graba `completedAt`, se navega al panel.
5. Reiniciar: el asistente ya no aparece.
6. Ajustes → «Repetir la configuración inicial»: reabre el asistente con los valores actuales, sin borrar datos.
7. Simular instalación previa (una clave de `settings` guardada, `completedAt` nulo): al arrancar se marca completada sin mostrar el asistente.
8. `historias.md` regenerado con `pnpm docs:build`; US-002 marcada como cubierta en `docs/user-stories.md`.

## Definición de terminado (cada pantalla, `docs/ui-design.md` §8)

Antes de dar por cerrado un PR que toca una pantalla, pasar su checklist §8: solo tokens, claro y oscuro, ventana mínima sin recortes, textos en es y en, cinco estados diseñados, datos ausentes como «No disponible», teclado y foco, acciones con carga confirmadas, sin permisos ni comandos nuevos, verificada a 1024×560 y 1280×720, verificada con un valor ausente en cada métrica, verificada con acento del sistema claro en los dos temas, verificada con 20 discos y 5.000 eventos, cero literales de interfaz.
