import adapter from "@sveltejs/adapter-static";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

/** SvelteKit compilado a estático: no hay servidor, la app vive dentro de Tauri (ADR-014).
 *  `ssr = false` y `prerender = true` se declaran en src/routes/+layout.ts. */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter({ fallback: "index.html", precompress: false, strict: false }),
    alias: {
      $lib: "src/lib",
      "$design-system": "src/design-system"
    }
  }
};

export default config;
