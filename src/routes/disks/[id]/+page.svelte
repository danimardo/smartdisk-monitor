<script lang="ts">
  /** Detalle de disco (US-012, US-020). El disco llega listo desde `load`. La composición aprobada
   *  está en el boceto v2 y se implementa en la versión 0.2. */
  import { EmptyState } from "$lib/components";
  import { t } from "$lib/i18n";
  import type { PageData } from "./$types";

  let { data }: { data: PageData } = $props();
</script>

<EmptyState kind="empty" title={data.disk.alias ?? data.disk.model} body={t("common.notImplemented")} />
