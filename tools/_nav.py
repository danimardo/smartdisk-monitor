import io

def fix(rel, pairs):
    for base in ("src/lib", "Design-system/src/lib"):
        p = f"{base}/{rel}"
        s = io.open(p, encoding="utf-8").read()
        for a, b in pairs:
            assert a in s, f"{p}: {a[:60]}"
            s = s.replace(a, b)
        io.open(p, "w", encoding="utf-8", newline="\n").write(s)
        print("ok", p)

# --- DiskCard: de botón con callback a enlace real ---
fix("components/DiskCard.svelte", [
    ("  let { disk = null as DiskSummary | null, onopen = undefined } = $props();",
     """  /** `href` en vez de un callback de navegación: un enlace real conserva ctrl+clic, clic central,
   *  menú contextual, foco y el anuncio como enlace de un lector de pantalla (constitución §XIV).
   *  Sin `href` la tarjeta se renderiza como bloque no interactivo, que es lo correcto cuando no
   *  lleva a ninguna parte. */
  let { disk = null as DiskSummary | null, href = undefined as string | undefined } = $props();"""),
    ('''  <button class="text-left" onclick={() => onopen?.(disk.id)} aria-label={t("disk.open", { name: disk.alias ?? disk.model })}>''',
     '''  <svelte:element
    this={href ? "a" : "div"}
    href={href || undefined}
    class="block text-left"
    aria-label={href ? t("disk.open", { name: disk.alias ?? disk.model }) : undefined}
  >'''),
])

s = io.open("src/lib/components/DiskCard.svelte", encoding="utf-8").read()
cierre_viejo = "  </button>"
assert cierre_viejo in s, "no encuentro el cierre del botón de DiskCard"
for base in ("src/lib", "Design-system/src/lib"):
    p = f"{base}/components/DiskCard.svelte"
    t = io.open(p, encoding="utf-8").read()
    t = t.replace("  </button>", "  </svelte:element>")
    io.open(p, "w", encoding="utf-8", newline="\n").write(t)
    print("ok cierre", p)

# --- Sidebar: secciones y discos como enlaces ---
fix("components/Sidebar.svelte", [
    ("""    sections = [] as { id: string; label: string; badge?: number | null }[],""",
     """    /** Cada sección lleva su `href`: la navegación se hace con enlaces reales, no con callbacks
     *  (constitución §XIV). */
    sections = [] as { id: string; label: string; href: string; badge?: number | null }[],"""),
    ("""    disks = [] as DiskSummary[],""",
     """    disks = [] as DiskSummary[],
    /** Construye el destino de cada disco. Devuelve el href, no navega. */
    diskHref = ((id: string) => `/disks/${id}`) as (id: string) => string,"""),
    ('''      <button class={row(active === s.id)} aria-current={active === s.id} onclick={() => onselect?.(s.id)}>''',
     '''      <a class={row(active === s.id)} href={s.href} aria-current={active === s.id ? "page" : undefined}>'''),
    ('''        <button class={row(activeDiskId === d.id)} aria-current={activeDiskId === d.id} onclick={() => onselectdisk?.(d.id)}>''',
     '''        <a class={row(activeDiskId === d.id)} href={diskHref(d.id)} aria-current={activeDiskId === d.id ? "page" : undefined}>'''),
])
