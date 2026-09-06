# 00b · Tipografía: qué empaquetar y cómo

Complementa `00-tokens.md` §3. Responde al punto 2 de tu revisión.

## No puedo adjuntar los binarios, pero aquí está todo lo necesario

Los `.woff2` no van dentro de esta entrega: son binarios que deben venir de la fuente oficial, con su
hash verificable, no de un ZIP de diseño. Lo que sí va es de dónde sacarlos, qué licencia tienen y cómo
generarlos exactamente.

## Bricolage Grotesque

- **Origen oficial:** `https://github.com/ateliertriay/bricolage` (Atelier Triay, Mathieu Triay).
- **Licencia:** SIL Open Font License 1.1. Permite empaquetarla en un producto comercial y de escritorio.
  Obligaciones prácticas: conservar el fichero `OFL.txt` junto a la fuente, mantener el aviso de copyright
  («Copyright 2022 The Bricolage Grotesque Project Authors»), y **no** vender la fuente por separado.
  Si en algún momento se modifica el binario, el nombre reservado no se puede reutilizar.
- **Es una fuente variable**, con ejes de peso, anchura y tamaño óptico. Eso cambia la recomendación:

### Recomendación: un solo woff2 variable, no tres estáticos

En `00-tokens.md` puse «pesos 500-700 estáticos». Con una variable sale mejor: **un** fichero con el eje
de peso recortado a 500-700 pesa menos que tres estáticos y da el 600 exacto.

```bash
pip install fonttools brotli

# 1) recortar el eje de peso y fijar los otros dos (no usamos anchura ni óptico variables)
fonttools varLib.instancer BricolageGrotesque[opsz,wdth,wght].ttf \
  wght=500:700 wdth=100 opsz=14 \
  -o bricolage-var.ttf

# 2) subconjunto latino + los signos que realmente usamos en cifras
pyftsubset bricolage-var.ttf \
  --unicodes="U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+2000-206F,U+2070,U+2074-2079,U+2080-2089,U+20AC,U+2122,U+2190-2193,U+2212,U+00B0,U+00B7,U+2026" \
  --layout-features="kern,liga,tnum,frac" \
  --flavor=woff2 --output-file=bricolage-500-700.woff2
```

`U+00B0` es el grado (°), `U+00B7` el punto medio (·) y `tnum` las cifras tabulares: los tres son
imprescindibles en esta interfaz y es fácil que un subconjunto por defecto se los coma.

### `@font-face`

Va en `tokens.css`, junto al de Instrument Sans. Es lo único que se añade al fichero además de los tokens.

```css
@font-face {
  font-family: "Bricolage Grotesque";
  src: url("../assets/fonts/bricolage-500-700.woff2") format("woff2-variations");
  font-weight: 500 700;
  font-style: normal;
  font-display: swap;
  unicode-range: U+0000-00FF, U+00B0, U+00B7, U+2000-206F, U+2026, U+20AC, U+2212;
}
```

Y **fuera** el `@import` de Google Fonts que hay hoy en `tokens.css`: en una aplicación de escritorio no
debe haber ninguna petición de red para pintar la interfaz. Lo mismo aplica a Instrument Sans, que hoy
también se carga por `@import`; conviene empaquetarla en el mismo movimiento (misma licencia OFL, origen
`https://github.com/Instrument/instrument-sans`).

### Si prefieres no empaquetar una segunda familia

Alternativa razonable, y la digo porque el coste no es nulo: usar **Instrument Sans 600** también para las
cifras de display, con `letter-spacing: -0.045em` y `font-feature-settings: "tnum"`. Se pierde el carácter
condensado de las cifras grandes —que es parte del atractivo de la dirección 2b— pero no se pierde nada
funcional, y ahorras ~40 KB y una licencia que gestionar.

Es tu decisión de ingeniería; el sistema funciona igual. Si eliges esta vía, `--sdm-font-display` pasa a
valer lo mismo que `--sdm-font-sans` y **no hay que tocar ni un componente**: la clase `.sdm-display`
sigue siendo el único punto de cambio.

## Ubicación en el repositorio

```
src/assets/fonts/
  bricolage-500-700.woff2
  instrument-sans-400-600.woff2
  OFL-bricolage.txt
  OFL-instrument-sans.txt
```

Y una línea en el diálogo Acerca de o en un `TERCEROS.md`: «Bricolage Grotesque © 2022 The Bricolage
Grotesque Project Authors, SIL OFL 1.1» más la equivalente de Instrument Sans. Con eso la atribución OFL
queda cubierta.
