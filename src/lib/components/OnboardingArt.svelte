<script lang="ts">
  /** Ilustración plana del asistente inicial (US-002), estilo «Corporate Memphis / Alegría»
   *  adaptado a la «escena de datos» de v3: formas geométricas rotundas, paleta cerrada y sin
   *  figuras humanas — el motivo es siempre el hardware y su vigilancia.
   *
   *  Es **decorativa**: sale `aria-hidden` y sin nombre accesible (el texto de cada paso ya dice
   *  todo). Como cualquier pieza del sistema, solo usa `currentColor` y `var(--sdm-*)`, así que
   *  funciona en tema claro y oscuro sin una sola condicional. Las coordenadas y los `stroke-width`
   *  son geometría del dibujo, no valores de tema (misma regla que el sprite de `IconSprite`).
   *
   *  Cinco escenas, una por paso: `welcome`, `disks`, `alerts`, `ai`, `done`. */
  type Escena = "welcome" | "disks" | "alerts" | "ai" | "done";

  let {
    name,
    /** Ancho en px; el alto sale de la proporción 8:5 del `viewBox`. */
    width = 232
  }: { name: Escena; width?: number } = $props();

  const height = $derived(Math.round((width * 150) / 240));
</script>

<svg {width} {height} viewBox="0 0 240 150" fill="none" aria-hidden="true" focusable="false" class="block">
  {#if name === "welcome"}
    <!-- Vigilancia serena: escudo sobre una pila de discos, con un latido que lo cruza. -->
    <ellipse cx="120" cy="118" rx="70" ry="15" fill="var(--sdm-glass-3)" />
    <ellipse cx="120" cy="104" rx="58" ry="12.5" fill="var(--sdm-accent-soft)" />
    <path
      d="M120 34c14 7 28 10 28 10v22c0 24-16 39-28 45-12-6-28-21-28-45V44s14-3 28-10Z"
      fill="var(--sdm-accent-soft)"
      stroke="var(--sdm-accent)"
      stroke-width="4"
      stroke-linejoin="round"
    />
    <path
      d="M92 82h13l6-13 9 26 6-13h13"
      stroke="var(--sdm-ok)"
      stroke-width="4"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
    <circle
      cx="182"
      cy="46"
      r="7"
      fill="var(--sdm-accent-soft)"
      stroke="var(--sdm-accent)"
      stroke-width="3"
    />
    <circle cx="58" cy="66" r="5" fill="var(--sdm-accent)" />
    <path d="M196 96l4 8 8 4-8 4-4 8-4-8-8-4 8-4Z" fill="var(--sdm-accent-soft)" />
    <circle cx="44" cy="104" r="4" fill="var(--sdm-accent-soft)" />
  {:else if name === "disks"}
    <!-- Tus discos: tres unidades como bloques, con su piloto, y una lupa que las revisa. -->
    <ellipse cx="120" cy="128" rx="78" ry="12" fill="var(--sdm-glass-3)" />
    <rect
      x="46"
      y="70"
      width="42"
      height="52"
      rx="9"
      fill="var(--sdm-accent-soft)"
      stroke="var(--sdm-accent)"
      stroke-width="3.5"
    />
    <rect
      x="100"
      y="46"
      width="42"
      height="76"
      rx="9"
      fill="var(--sdm-accent-soft)"
      stroke="var(--sdm-accent)"
      stroke-width="3.5"
    />
    <rect
      x="154"
      y="86"
      width="42"
      height="36"
      rx="9"
      fill="var(--sdm-glass-3)"
      stroke="var(--sdm-unknown)"
      stroke-width="3.5"
    />
    <circle cx="67" cy="84" r="4.5" fill="var(--sdm-ok)" />
    <circle cx="121" cy="60" r="4.5" fill="var(--sdm-ok)" />
    <circle cx="175" cy="100" r="4.5" fill="var(--sdm-unknown)" />
    <circle cx="150" cy="52" r="20" fill="none" stroke="var(--sdm-accent)" stroke-width="4" />
    <path d="M164 66l16 16" stroke="var(--sdm-accent)" stroke-width="5" stroke-linecap="round" />
    <path d="M40 36l3.5 7 7 3.5-7 3.5-3.5 7-3.5-7-7-3.5 7-3.5Z" fill="var(--sdm-accent-soft)" />
    <circle cx="206" cy="52" r="4" fill="var(--sdm-accent)" />
  {:else if name === "alerts"}
    <!-- Cuánto avisar: una campana que suena, con sus ondas a ambos lados. -->
    <ellipse cx="120" cy="126" rx="60" ry="12" fill="var(--sdm-glass-3)" />
    <path
      d="M120 34a9 9 0 0 1 9 9c17 5 21 21 21 36 0 11 4 17 8 21H82c4-4 8-10 8-21 0-15 4-31 21-36a9 9 0 0 1 9-9Z"
      fill="var(--sdm-accent-soft)"
      stroke="var(--sdm-accent)"
      stroke-width="4"
      stroke-linejoin="round"
    />
    <path d="M111 122a9 9 0 0 0 18 0" stroke="var(--sdm-accent)" stroke-width="4" stroke-linecap="round" />
    <circle cx="120" cy="98" r="4.5" fill="var(--sdm-accent)" />
    <path
      d="M64 60c-6 8-8 20-5 32"
      stroke="var(--sdm-accent)"
      stroke-width="3.5"
      stroke-linecap="round"
      opacity="0.8"
    />
    <path
      d="M50 52c-9 12-11 30-6 46"
      stroke="var(--sdm-accent)"
      stroke-width="3.5"
      stroke-linecap="round"
      opacity="0.4"
    />
    <path
      d="M176 60c6 8 8 20 5 32"
      stroke="var(--sdm-accent)"
      stroke-width="3.5"
      stroke-linecap="round"
      opacity="0.8"
    />
    <path
      d="M190 52c9 12 11 30 6 46"
      stroke="var(--sdm-accent)"
      stroke-width="3.5"
      stroke-linecap="round"
      opacity="0.4"
    />
    <path d="M150 34l3 6 6 3-6 3-3 6-3-6-6-3 6-3Z" fill="var(--sdm-accent-soft)" />
  {:else if name === "ai"}
    <!-- Explícamelo: un disco cuyo detalle técnico sale traducido a lenguaje llano en un globo,
         con la chispa de la ayuda y la nube que recuerda que la consulta viaja fuera. -->
    <ellipse cx="120" cy="128" rx="76" ry="12" fill="var(--sdm-glass-3)" />
    <rect
      x="34"
      y="72"
      width="58"
      height="46"
      rx="10"
      fill="var(--sdm-accent-soft)"
      stroke="var(--sdm-accent)"
      stroke-width="3.5"
    />
    <circle cx="49" cy="86" r="4.5" fill="var(--sdm-ok)" />
    <path
      d="M112 26h72a16 16 0 0 1 16 16v32a16 16 0 0 1-16 16h-50l-40 16 24-16h-6a16 16 0 0 1-16-16V42a16 16 0 0 1 16-16Z"
      fill="var(--sdm-accent-soft)"
      stroke="var(--sdm-accent)"
      stroke-width="4"
      stroke-linejoin="round"
    />
    <path
      d="M114 44h68M114 58h54M114 72h36"
      stroke="var(--sdm-accent)"
      stroke-width="4"
      stroke-linecap="round"
    />
    <path
      d="M128 14a10 10 0 0 1 19-2 7 7 0 0 1 2 13h-22a8 8 0 0 1 1-11Z"
      fill="none"
      stroke="var(--sdm-accent)"
      stroke-width="3"
      stroke-linejoin="round"
      opacity="0.5"
    />
    <path d="M206 24l4 9 9 4-9 4-4 9-4-9-9-4 9-4Z" fill="var(--sdm-accent-soft)" />
    <circle cx="30" cy="52" r="4" fill="var(--sdm-accent)" />
    <circle cx="120" cy="112" r="4" fill="var(--sdm-accent-soft)" />
    <path d="M214 96l3 6 6 3-6 3-3 6-3-6-6-3 6-3Z" fill="var(--sdm-accent-soft)" />
  {:else}
    <!-- Todo listo: sello de conformidad y la primera lectura arrancando. -->
    <ellipse cx="120" cy="128" rx="70" ry="12" fill="var(--sdm-glass-3)" />
    <circle cx="120" cy="74" r="42" fill="var(--sdm-ok-soft)" stroke="var(--sdm-ok)" stroke-width="4" />
    <path
      d="M102 74l12 12 24-26"
      stroke="var(--sdm-ok)"
      stroke-width="6"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
    <path
      d="M60 112h16l8-14 10 22 7-12h62"
      stroke="var(--sdm-accent)"
      stroke-width="3.5"
      stroke-linecap="round"
      stroke-linejoin="round"
      opacity="0.55"
    />
    <circle cx="176" cy="44" r="6" fill="var(--sdm-accent)" />
    <rect x="52" y="52" width="10" height="10" rx="3" fill="var(--sdm-accent-soft)" />
    <path d="M188 92l3.5 7 7 3.5-7 3.5-3.5 7-3.5-7-7-3.5 7-3.5Z" fill="var(--sdm-ok-soft)" />
    <path d="M64 34c4 2 8 2 12 0" stroke="var(--sdm-accent)" stroke-width="3.5" stroke-linecap="round" />
  {/if}
</svg>
