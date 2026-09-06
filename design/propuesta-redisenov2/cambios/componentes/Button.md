# Button

## Cambio

Solo uno, y es de corrección: la variante `primary` tiene que escribir **`text-fg-onAccent`**, no
`text-white`.

En la paleta Ciruela el acento del tema oscuro es claro (`#c79aec`). Texto blanco encima da **2,27:1**:
ilegible e incumple AA con holgura. `--sdm-on-accent` vale `#20132a` en oscuro, que da **7,80:1**.

```diff
- primary: "… text-white bg-[linear-gradient(180deg,var(--sdm-accent-hi),var(--sdm-accent))] …"
+ primary: "… text-fg-onAccent bg-[linear-gradient(180deg,var(--sdm-accent-hi),var(--sdm-accent))] …"
```

Hay que revisar el mismo error en todo sitio que pinte sobre el acento:

- el cuadrado de icono de la cabecera de `DiskCard` (fondo = color de estado);
- el logotipo del riel;
- la casilla marcada del asistente inicial;
- el relleno de `ProgressBar` en modo `display` — **excepción**: ahí no hay texto encima, y el filo
  interior sigue siendo `rgba(255,255,255,.35)` en los dos temas, porque es un brillo, no tinta.
- el punto del `Switch` activo — **excepción**: es blanco en los dos temas, no es texto.

## Sin cambios

Variantes, tamaños, radios, alturas, el `active:scale-[0.98]`, el degradado vertical, el filo interior,
`disabledReason` obligatorio y la regla de una sola `primary` por pantalla.
