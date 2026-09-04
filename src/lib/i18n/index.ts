/** Barrel del módulo de i18n.
 *
 *  La implementación vive en `i18n.svelte.ts` porque usa runes (`$state`), y Svelte 5 solo las
 *  compila en ficheros `.svelte.ts`. Este barrel existe para que el resto del código pueda seguir
 *  importando de `$lib/i18n` sin conocer ese detalle.
 */
export { i18n, t, tp, type Locale } from "./i18n.svelte";
