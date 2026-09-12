# Cómo entregar la propuesta de rediseño

Para que el rediseño se pueda **implementar sin interpretar**, la propuesta tiene que llegar en el
formato que se describe aquí. Cuanto más se acerque, menos ida y vuelta hará falta.

El objetivo no es un PDF bonito: es un documento del que se pueda sacar, línea a línea, qué token
cambia, qué componente se recompone y qué marcado nuevo hace falta.

---

## 1. Estructura de la entrega

Devuelve una carpeta (o ZIP) con:

```
propuesta-rediseno/
  RESUMEN.md                  <- qué cambia y por qué, en 1–2 páginas
  cambios/
    00-tokens.md              <- todos los cambios de tokens, juntos
    01-panel-general.md       <- un fichero por pantalla
    02-alertas.md
    ...
    09-onboarding.md
    componentes/
      Button.md               <- solo los componentes que cambian
      Sidebar.md
      ...
  mockups/
    panel-general-claro.png / .html
    panel-general-oscuro.png / .html
    ...
```

Los mockups en **HTML/CSS** son muy preferibles a PNG o Figma: se comparan directamente con los
snapshots de `salida/html/` y se puede ver qué clases y qué variables han cambiado. Si usas Figma,
exporta además los valores (spacing, color, radios) como texto, no solo la imagen.

---

## 2. `RESUMEN.md`

- Los 3–7 cambios de fondo (p. ej. "más aire vertical en las tarjetas", "jerarquía tipográfica más
  marcada en los títulos de sección", "el estado global pasa de la Toolbar a la Sidebar").
- Para cada uno: **problema observado** en las capturas actuales → **cambio propuesto** →
  **por qué es mejor**.
- Qué **no** se toca, para dejarlo claro.

---

## 3. `cambios/00-tokens.md`

La tabla completa de tokens que cambian o se añaden. Un token es un valor de `tokens-referencia.css`
(prefijo `--sdm-`). **Nada de valores sueltos fuera de esta tabla.**

| Token | Ahora (claro / oscuro) | Propuesto (claro / oscuro) | Dónde afecta | Motivo |
|---|---|---|---|---|
| `--sdm-space-card` | `16px` | `20px` | padding de todas las `Card` y `DiskCard` | … |
| `--sdm-nuevo-...` | — (nuevo) | `… / …` | … | … |

Reglas:
- Si un color nuevo no llega a contraste **AA** sobre el material compuesto (no sobre el fondo
  sólido), no vale. Indica el ratio si lo has medido.
- Todo token nuevo necesita **valor en claro y valor en oscuro**, aunque coincidan.
- No propongas cambiar la familia tipográfica ni pesos por encima de 600 (no existe la negrita 700).
- Los radios salen de la escala `--sdm-radius-*` (card 18 / inner 13 / nav / pill). Si necesitas
  otro escalón, proponlo como token, no como número.

---

## 4. `cambios/NN-<pantalla>.md`

Un fichero por pantalla (las 8 de `inventario-pantallas.md`, más `onboarding`). Cada uno:

### 4.1. Diagnóstico
Qué falla hoy en esa pantalla, referido a la captura concreta (`salida/capturas/<fichero>.png`).

### 4.2. Cambios de distribución
Descríbelos en términos de caja: qué se mueve, qué cambia de tamaño, qué se agrupa. Ejemplo:

> La rejilla de `DiskCard` pasa de columna mínima 460 px a 380 px y de `gap` 20 → 16.
> El bloque de tres métricas pasa de fila a rejilla 2×2 cuando la tarjeta baja de 420 px.

### 4.3. Cambios por componente
Para cada componente afectado: qué cambia en su estructura interna, sus estados o su espaciado.
Si es solo tokens, remite a `00-tokens.md`. Si cambia el marcado, ponlo (ver §5).

### 4.4. Estados
Confirma que los estados de esa pantalla siguen cubiertos y cómo quedan: cargando, vacío, no
compatible, error de fuente, dato obsoleto (`ui-design.md` §8). Si cambias el estado vacío, adjunta
mockup.

### 4.5. Claro y oscuro
Mockup en los dos temas, o nota explícita de que el cambio es puramente de token y el tema se
resuelve solo.

### 4.6. Ventana mínima
Cómo queda a 1024 × 560. Si algo se reordena o se colapsa a ese ancho, descríbelo.

---

## 5. `cambios/componentes/<Componente>.md`

Solo para los componentes cuyo **marcado o comportamiento** cambia (no los que solo heredan un
token nuevo). Incluye:

- **Antes / después** de la estructura (árbol de elementos o fragmento de plantilla).
- Props nuevas o cambiadas, con su valor por defecto. Todo callback es opcional: el componente
  tiene que poder usarse sin él.
- Clases de utilidad y variables `--sdm-*` empleadas. Cero literales.
- Si propones un **componente nuevo**: justifícalo contra `ui-design.md` §3 (por qué ninguno del
  catálogo sirve) y da su API completa y sus estados.

---

## 6. Texto e idioma

Si el rediseño cambia algún texto visible (etiquetas, títulos, textos de estado vacío), lista los
cambios en `RESUMEN.md` como tabla `clave → es → en`. No hace falta que toques los diccionarios;
solo que quede claro qué frase nueva va en cada sitio, en los dos idiomas.

---

## 7. Lista de comprobación antes de entregar

- [ ] Ningún color, radio, sombra o tamaño escrito a mano fuera de `00-tokens.md`.
- [ ] Todo token nuevo tiene valor claro y oscuro.
- [ ] Cada pantalla tiene mockup en claro y en oscuro (o nota de "solo tokens").
- [ ] Los cinco estados diseñados siguen cubiertos en cada pantalla.
- [ ] Nada se recorta a 1024 × 560.
- [ ] No se apilan materiales (una tarjeta no contiene otra tarjeta).
- [ ] El color siempre va con texto o icono.
- [ ] Propuesta para `onboarding`, que hoy no existe.

---

## Qué haré yo al recibirlo

1. Reviso la propuesta contra la constitución del proyecto y `ui-design.md`; si algo choca, lo
   señalo antes de tocar código.
2. Llevo `00-tokens.md` a `src/design-system/tokens.css` y su `tokens.json`.
3. Aplico los cambios de componente y de pantalla, uno a uno, con sus pruebas.
4. Verifico claro/oscuro, ventana mínima y accesibilidad, y paso `pnpm verify` (que comprueba que
   no se han colado valores literales).

Cuanto más detallada sea la sección de tokens y la de "antes/después" de cada componente, más
directo es ese proceso.
