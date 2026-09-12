# Textos para la landing de SmartDisk Monitor

Copy de presentación y descarga, pensado para una página de aterrizaje (landing) — no para el
`README.md` del repositorio, que ya cumple su propio papel técnico. Aquí el tono es de venta:
más directo, más corto por bloque, orientado a que alguien que no conoce la aplicación entienda en
segundos qué hace y se descargue el instalador.

**Es solo contenido.** No hay maquetación ni HTML: eso lo decide quien construya la página real
(sitio propio, GitHub Pages, Webflow...). Cada fichero es un bloque de la página, en el orden en que
aparecerían.

## Punto de partida

- La aplicación es **gratuita y de código abierto** (licencia MIT del código propio). No hay precio,
  planes ni versión de pago — «vender» aquí significa convencer de que merece la pena instalarla y
  descargarla, no cobrar por ella.
- Fuentes usadas: [`README.md`](../../README.md), [`docs/product-specification.md`](../../docs/product-specification.md),
  las capturas de [`docs/screenshots/`](../../docs/screenshots/) y el tono ya explorado en
  [`Videos/Prompts.md`](../Videos/Prompts.md).
- Repositorio: https://github.com/danimardo/smartdisk-monitor · Autor: Daniel Diez Mardomingo.

## Ficheros

| Fichero | Bloque de la página |
|---|---|
| [`01-portada.md`](01-portada.md) | Cabecera: titular, subtitular, CTA principal y sello de confianza |
| [`02-funcionalidades-y-privacidad.md`](02-funcionalidades-y-privacidad.md) | Funcionalidades destacadas y bloque de privacidad |
| [`03-capturas-y-demo.md`](03-capturas-y-demo.md) | Un bloque de texto por captura de `docs/screenshots/`, más nota sobre los vídeos |
| [`04-descarga-requisitos-faq.md`](04-descarga-requisitos-faq.md) | CTA de descarga, requisitos, avisos esperados (SmartScreen, Defender) y FAQ |

## Qué falta antes de publicar nada

- **Maquetación real.** Si se decide construir la página, conviene pasar por la skill `design` del
  proyecto para el boceto visual antes de escribir código.
- **Revisión humana del tono de venta.** Este texto es más «vendedor» que el resto del repositorio a
  propósito; si no encaja con la voz que se quiera dar de cara al público, es el sitio a ajustar.
- **Enlace de descarga real.** Se usa `https://github.com/danimardo/smartdisk-monitor/releases` en
  todo el texto, igual que en el `README.md`. Si la landing acaba viviendo en un dominio propio con
  redirección propia, actualizar los enlaces.
