# Tipografía empotrada

`tokens.css` declara la familia Instrument Sans sobre los ficheros de esta carpeta. La aplicación no
descarga tipografías: la especificación (§11) y el ADR-018 prohíben cualquier petición de red
durante el funcionamiento normal, y un equipo sin salida a Internet debe renderizar exactamente
igual que uno conectado.

## Ficheros

| Fichero | Subconjunto | Tamaño | SHA-256 |
|---|---|---|---|
| `InstrumentSans-latin.woff2` | latin | 30.092 B | `2ee17598a98d8a59e4df8152d015bec9ab8e4d5672cc0ab42bef806b568e3971` |
| `InstrumentSans-latin-ext.woff2` | latin-ext | 11.144 B | `c4fcfea41f2c1cfeea9211fa43679845454a1d0e0d7e95e069c7e73c4ae302d2` |
| `OFL.txt` | — | 4.403 B | licencia, sin modificar |

Son ficheros **variables** en el eje `wght` (400–700), con el eje `wdth` fijado en 100. `tokens.css`
declara solo `400 600` a propósito: si alguien pidiera 700, el navegador lo limita a 600 en lugar de
sintetizar una negrita falsa. El peso máximo de v2 es 600 (`AGENTS.md` §2.bis).

## Por qué dos ficheros

El proyecto original publica la fuente subseteada, igual que la sirve Google Fonts. `latin` cubre
por completo el español y el inglés de la interfaz —vocales acentuadas, `ñ`, `ü`, `¿`, `¡`, `°`, `×`,
`·`, `•`— y es el único que se carga en uso normal. `latin-ext` solo se decodifica si aparece un
carácter de su rango, cosa que puede ocurrir con el texto original de un evento de Windows, que
llega en el idioma del sistema. Al ser ficheros locales, tener los dos no cuesta nada; el
`unicode-range` evita procesar el que no hace falta.

## Procedencia

- Proyecto: Instrument Sans, de Rodrigo Fuenzalida y Jordan Egstad.
- Repositorio: https://github.com/Instrument/instrument-sans
- Ficheros obtenidos de: `https://fonts.gstatic.com/s/instrumentsans/v4/…` (versión **v4** del
  catálogo de Google Fonts, que es quien publica los `.woff2` ya subseteados).
- `OFL.txt` obtenida de: https://github.com/google/fonts/blob/main/ofl/instrumentsans/OFL.txt
- Copyright 2022 The Instrument Sans Project Authors.
- Licencia: SIL Open Font License 1.1.

## Obligaciones

1. `OFL.txt` viaja con la aplicación, sin modificar. **Ya está en esta carpeta.**
2. La fuente está registrada en `THIRD_PARTY_NOTICES.md` con su versión y sus hashes.
3. No se renombra la familia: la OFL solo obliga a cambiar el nombre si se modifica el fichero, y
   aquí se redistribuye tal cual.
4. Si se actualiza la fuente, se actualizan los hashes de esta tabla y los del aviso de terceros.

## Verificación

```sh
sha256sum -c <<'EOF'
2ee17598a98d8a59e4df8152d015bec9ab8e4d5672cc0ab42bef806b568e3971 *InstrumentSans-latin.woff2
c4fcfea41f2c1cfeea9211fa43679845454a1d0e0d7e95e069c7e73c4ae302d2 *InstrumentSans-latin-ext.woff2
EOF
```

## Comprobación visual

`tools/font-check.html` renderiza la fuente en ambos subconjuntos y los tres pesos. Necesita un
servidor, porque `file://` no carga `.woff2`:

```sh
python -m http.server 8731 --bind 127.0.0.1     # desde la raíz del repositorio
# abrir http://127.0.0.1:8731/tools/font-check.html
```

Verificado el 2026-09-04: los dos `@font-face` cargan (`document.fonts.check` a `true`), los
acentos españoles, `°`, `¿`, `¡`, `«»`, `×` y `·` renderizan con Instrument Sans, el subconjunto
`latin-ext` entra cuando toca, y los pesos 600 y 700 miden exactamente lo mismo: el clamp de
`tokens.css` impide la negrita sintética.

## Por qué falla la compilación sin estos ficheros

La compilación debe fallar si falta cualquiera de los dos `.woff2`: sin ellos la aplicación cae a
`Segoe UI`, la métrica cambia, los bocetos aprobados dejan de ser fieles y nadie se entera
(`docs/engineering-conventions.md` §3).
