<script lang="ts">
  /** Sprite de iconos de línea (v3, ADR-034). Define los 21 `<symbol id="i-…">` que `<Icon>`
   *  referencia con `<use>`. Se monta **una sola vez** en `src/routes/+layout.svelte`, fuera de
   *  `AppShell`, para que los iconos también estén disponibles en rutas sin chrome como el asistente
   *  inicial (`/onboarding`): antes vivía dentro de `AppShell` y en esa ruta los `<use>` no
   *  resolvían y los iconos salían en blanco.
   *
   *  Los `stroke-width` literales son la geometría de cada símbolo, no valores de tema;
   *  `--sdm-icon-stroke` (1,7) queda como valor de referencia en `tokens.css`. Origen:
   *  design/propuesta-redisenov2/mockups/icons-sprite.svg (sin el bloque `<metadata>` de procedencia). */
</script>

<svg width="0" height="0" style="position: absolute; overflow: hidden" aria-hidden="true">
  <defs>
    <symbol id="i-temp" viewBox="0 0 24 24"
      ><path
        d="M12 3.5a2.5 2.5 0 0 0-2.5 2.5v7.4a4.5 4.5 0 1 0 5 0V6A2.5 2.5 0 0 0 12 3.5Z"
        fill="none"
        stroke="currentColor"
        stroke-width="1.7"
        stroke-linejoin="round"
      ></path><path d="M12 8.5v6.2" stroke="currentColor" stroke-width="1.7" stroke-linecap="round"
      ></path><circle cx="12" cy="17.4" r="1.7" fill="currentColor"></circle></symbol
    >
    <symbol id="i-wear" viewBox="0 0 24 24"
      ><path
        d="M4 17a8 8 0 1 1 16 0"
        fill="none"
        stroke="currentColor"
        stroke-width="1.7"
        stroke-linecap="round"
      ></path><path d="M12 17l4.2-5" stroke="currentColor" stroke-width="1.7" stroke-linecap="round"
      ></path><circle cx="12" cy="17" r="1.4" fill="currentColor"></circle></symbol
    >
    <symbol id="i-pulse" viewBox="0 0 24 24"
      ><path
        d="M2.5 12.5h4l2-5 3.5 10 3-7 2 2h4.5"
        fill="none"
        stroke="currentColor"
        stroke-width="1.7"
        stroke-linecap="round"
        stroke-linejoin="round"
      ></path></symbol
    >
    <symbol id="i-clock" viewBox="0 0 24 24"
      ><circle cx="12" cy="12" r="8.4" fill="none" stroke="currentColor" stroke-width="1.7"></circle><path
        d="M12 7.6V12l3.2 2"
        fill="none"
        stroke="currentColor"
        stroke-width="1.7"
        stroke-linecap="round"
      ></path></symbol
    >
    <symbol id="i-nvme" viewBox="0 0 24 24"
      ><rect
        x="3.2"
        y="7.5"
        width="17.6"
        height="9"
        rx="2"
        fill="none"
        stroke="currentColor"
        stroke-width="1.7"
      ></rect><path
        d="M7 16.5v2.6M11 16.5v2.6M15 16.5v2.6"
        stroke="currentColor"
        stroke-width="1.6"
        stroke-linecap="round"
      ></path><path d="M7.5 11.5h6" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"
      ></path></symbol
    >
    <symbol id="i-hdd" viewBox="0 0 24 24"
      ><circle cx="12" cy="12" r="8.4" fill="none" stroke="currentColor" stroke-width="1.7"></circle><circle
        cx="12"
        cy="12"
        r="2.4"
        fill="none"
        stroke="currentColor"
        stroke-width="1.6"
      ></circle><path d="M13.9 13.9l4 4" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"
      ></path></symbol
    >
    <symbol id="i-usb" viewBox="0 0 24 24"
      ><path d="M12 20.5V6" stroke="currentColor" stroke-width="1.7" stroke-linecap="round"></path><path
        d="M9.4 6.4L12 3.5l2.6 2.9"
        fill="none"
        stroke="currentColor"
        stroke-width="1.7"
        stroke-linejoin="round"
      ></path><path d="M12 14l-3.6-2.4V9" fill="none" stroke="currentColor" stroke-width="1.7"></path><circle
        cx="8.4"
        cy="8.2"
        r="1.5"
        fill="currentColor"
      ></circle><path d="M12 16.6l3.8-2.5v-2.4" fill="none" stroke="currentColor" stroke-width="1.7"
      ></path><rect x="14.3" y="9.2" width="3" height="2.6" fill="currentColor"></rect></symbol
    >
    <symbol id="i-shield" viewBox="0 0 24 24"
      ><path
        d="M12 3.4l6.6 2.4v5.5c0 4.2-2.8 7.4-6.6 9-3.8-1.6-6.6-4.8-6.6-9V5.8L12 3.4Z"
        fill="none"
        stroke="currentColor"
        stroke-width="1.7"
        stroke-linejoin="round"
      ></path><path
        d="M9.2 12.2l2.1 2.1 3.6-4"
        fill="none"
        stroke="currentColor"
        stroke-width="1.7"
        stroke-linecap="round"
        stroke-linejoin="round"
      ></path></symbol
    >
    <symbol id="i-alert" viewBox="0 0 24 24"
      ><path
        d="M12 4.4l8.2 14.2H3.8L12 4.4Z"
        fill="none"
        stroke="currentColor"
        stroke-width="1.7"
        stroke-linejoin="round"
      ></path><path d="M12 9.6v4.2" stroke="currentColor" stroke-width="1.7" stroke-linecap="round"
      ></path><circle cx="12" cy="16.4" r="1.1" fill="currentColor"></circle></symbol
    >
    <symbol id="i-flask" viewBox="0 0 24 24"
      ><path
        d="M9 3.5h6M10 3.5v5.2L5.6 17a2.4 2.4 0 0 0 2.1 3.5h8.6a2.4 2.4 0 0 0 2.1-3.5L14 8.7V3.5"
        fill="none"
        stroke="currentColor"
        stroke-width="1.7"
        stroke-linejoin="round"
      ></path><path d="M7.4 14.4h9.2" stroke="currentColor" stroke-width="1.6"></path></symbol
    >
    <symbol id="i-diskStack" viewBox="0 0 24 24"
      ><ellipse cx="12" cy="6.6" rx="7.6" ry="3.1" fill="none" stroke="currentColor" stroke-width="1.7"
      ></ellipse><path
        d="M4.4 6.6v5.2c0 1.7 3.4 3.1 7.6 3.1s7.6-1.4 7.6-3.1V6.6"
        fill="none"
        stroke="currentColor"
        stroke-width="1.7"
      ></path><path
        d="M4.4 11.8v5.2c0 1.7 3.4 3.1 7.6 3.1s7.6-1.4 7.6-3.1v-5.2"
        fill="none"
        stroke="currentColor"
        stroke-width="1.7"
      ></path></symbol
    >
    <symbol id="i-bolt" viewBox="0 0 24 24"
      ><path
        d="M13.4 3.2L6.2 13.4h4.6l-.8 7.4 7.6-10.6h-4.6l.4-7Z"
        fill="none"
        stroke="currentColor"
        stroke-width="1.7"
        stroke-linejoin="round"
      ></path></symbol
    >
    <symbol id="i-plug" viewBox="0 0 24 24"
      ><path d="M8 3.6v4.2M16 3.6v4.2" stroke="currentColor" stroke-width="1.7" stroke-linecap="round"
      ></path><path
        d="M5.4 7.8h13.2v3.4a6.6 6.6 0 0 1-13.2 0V7.8Z"
        fill="none"
        stroke="currentColor"
        stroke-width="1.7"
        stroke-linejoin="round"
      ></path><path d="M12 17.8v2.8" stroke="currentColor" stroke-width="1.7" stroke-linecap="round"
      ></path></symbol
    >
    <symbol id="i-check" viewBox="0 0 24 24"
      ><path
        d="M5 12.6l4.4 4.4L19 7.4"
        fill="none"
        stroke="currentColor"
        stroke-width="2.1"
        stroke-linecap="round"
        stroke-linejoin="round"
      ></path></symbol
    >
    <symbol id="i-tag" viewBox="0 0 24 24"
      ><path
        d="M11.4 3.6H20v8.6l-8.8 8.8-8.6-8.6 8.8-8.8Z"
        fill="none"
        stroke="currentColor"
        stroke-width="1.7"
        stroke-linejoin="round"
      ></path><circle cx="16.2" cy="7.8" r="1.5" fill="currentColor"></circle></symbol
    >
    <symbol id="i-close" viewBox="0 0 24 24"
      ><path
        d="M6 6l12 12M18 6L6 18"
        fill="none"
        stroke="currentColor"
        stroke-width="1.7"
        stroke-linecap="round"
      ></path></symbol
    >
    <symbol id="i-sparkles" viewBox="0 0 24 24"
      ><path
        d="M9.5 3c0 4.5 1 5.5 6.5 6.5-5.5 1-6.5 2-6.5 6.5 0-4.5-1-5.5-6.5-6.5C8.5 8.5 9.5 7.5 9.5 3Z"
        fill="currentColor"
        stroke="currentColor"
        stroke-width="1.1"
        stroke-linejoin="round"
      ></path><path
        d="M17.5 13c0 2 .5 2.5 3 3-2.5.5-3 1-3 3 0-2-.5-2.5-3-3 2.5-.5 3-1 3-3Z"
        fill="currentColor"
        stroke="currentColor"
        stroke-width="1.1"
        stroke-linejoin="round"
      ></path></symbol
    >
    <symbol id="i-minimize" viewBox="0 0 24 24"
      ><path d="M6 17h12" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round"
      ></path></symbol
    >
    <symbol id="i-maximize" viewBox="0 0 24 24"
      ><rect x="6" y="6" width="12" height="12" fill="none" stroke="currentColor" stroke-width="1.7"
      ></rect></symbol
    >
    <symbol id="i-restore" viewBox="0 0 24 24"
      ><rect x="9" y="5" width="9" height="9" fill="none" stroke="currentColor" stroke-width="1.7"
      ></rect><rect x="5" y="9" width="9" height="9" fill="none" stroke="currentColor" stroke-width="1.7"
      ></rect></symbol
    >
    <symbol id="i-sidebarToggle" viewBox="0 0 24 24"
      ><rect x="3.5" y="5" width="17" height="14" rx="2" fill="none" stroke="currentColor" stroke-width="1.7"
      ></rect><path d="M9 5v14" stroke="currentColor" stroke-width="1.7"></path></symbol
    >
  </defs>
</svg>
