# Contrato — `preview_informe_ia` (nuevo, sin red)

Devuelve, por disco incluido, el texto exacto y **ya anonimizado** que se enviaría al modelo, para
la vista previa que exige el principio XVI. **No toca la red** (análogo a `preview_diagnostic_zip`).

## Firma

```ts
invoke<PreviewInformeIaWire>("preview_informe_ia", {
  fromUtc: string,
  toUtc: string,
  deviceIds: string[] | null,
  alertLabels: Record<string, string> | null
})

type PreviewInformeIaWire = {
  discos: {
    deviceId: string;
    deviceLabel: string;              // para la persona (NO anonimizado)
    textoEnviado: string;             // system + user exactos, anonimizados y recortados
    fragmentos: FragmentoDudosoWire[];    // ya existe (spec 006): texto libre residual dudoso
    recortado: boolean;               // el payload superó el máximo y se truncó
  }[];
  redactedFields: string[];           // categorías anonimizadas (claves i18n: serial, computerName, userPaths, volumeLabel, sid, wwn, devicePath)
  totalLlamadas: number;              // = discos.length
};
```

## Comportamiento

1. Resuelve los discos (mismos que `export_report`).
2. Por cada disco: reúne alertas del intervalo (con `target_device_id == disco`), el contenido de
   los sucesos que las originaron, contadores SMART, resumen numérico de temperatura y actividad.
3. Compone `system` + `user` con `ia::componer_consulta` (variante informe).
4. **Anonimiza**: `Anonimizador::para_esta_maquina(series)` + `.con_etiqueta_volumen(...)` por cada
   volumen del inventario, luego `ia::redactar_identificadores`.
5. Recorta a `MAX_DETALLE_CHARS` (ya existe).
6. `barrer_texto_residual` sobre el texto anonimizado → `fragmentos` (omitido si
   `settings.ai.send_without_review`).
7. Devuelve todo junto; **nada sale del proceso**.

## Errores

| Código | Cuándo |
|---|---|
| `ia.no_key` | no hay clave configurada |
| `device.not_found` | un `deviceId` no existe |
| `ipc.schema_mismatch` | fechas no RFC3339 |

## Pruebas

- Un disco con un número de serie / etiqueta de volumen / nombre de equipo en su payload →
  aparecen como marcadores en `textoEnviado`, y sus categorías en `redactedFields`.
- `textoEnviado` de un disco **no** contiene datos de otro disco.
- `totalLlamadas == discos.length`.
- El comando no genera ninguna petición de red (comprobable en e2e con el monitor de red del IPC
  falso / en unit test porque no llama a `platform::ia_openrouter`).
