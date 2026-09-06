/** Panel general (US-012). El inventario lo carga el `onMount` del layout (una sola fuente, v3) y
 *  vive en el store; esta ruta solo aporta el título de la barra de herramientas y no vuelve a
 *  pedir nada. Las actualizaciones llegan por los eventos que empuja el backend (ADR-015). */
import type { PageLoad } from "./$types";

export const load: PageLoad = () => ({});
