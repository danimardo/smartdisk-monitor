import js from "@eslint/js";
import svelte from "eslint-plugin-svelte";
import globals from "globals";
import ts from "typescript-eslint";
import svelteParser from "svelte-eslint-parser";

/** Configuración de ESLint.
 *
 *  **Deliberadamente no solapa con `svelte-check`**, que ya cubre tipos y accesibilidad y que la
 *  constitución §XIII exige ejecutar con cero errores y cero avisos. Duplicar esas comprobaciones
 *  solo produciría ruido y dos sitios donde silenciar lo mismo.
 *
 *  Lo que ESLint aporta aquí y `svelte-check` no ve son las **promesas sin gestionar**: un `await`
 *  olvidado o un `.catch()` que falta. En una aplicación que se abre el lunes y sigue abierta el
 *  viernes (constitución §XIV), esa clase de fallo no revienta al momento: se acumula hasta que
 *  alguien reporta que "va lenta" y no hay forma de reproducirlo.
 */
export default ts.config(
  js.configs.recommended,
  ...ts.configs.recommendedTypeChecked,
  ...svelte.configs["flat/recommended"],

  {
    languageOptions: {
      globals: { ...globals.browser, ...globals.node },
      parserOptions: {
        projectService: true,
        extraFileExtensions: [".svelte"]
      }
    }
  },

  {
    rules: {
      /* --- Lo que justifica tener ESLint --- */

      // Una promesa sin gestionar en una app de larga ejecución no falla: se acumula.
      "@typescript-eslint/no-floating-promises": "error",
      // Pasar una función async donde se espera una síncrona: el error se pierde en silencio.
      "@typescript-eslint/no-misused-promises": "error",
      // Un `async` sin `await` suele ser una llamada que alguien olvidó esperar.
      "@typescript-eslint/require-await": "error",
      "@typescript-eslint/await-thenable": "error",

      // Código muerto: en un proyecto con catálogo cerrado, un import sin usar suele ser el
      // rastro de un refactor a medias.
      "@typescript-eslint/no-unused-vars": [
        "error",
        { argsIgnorePattern: "^_", varsIgnorePattern: "^_", caughtErrorsIgnorePattern: "^_" }
      ],

      // La constitución §XI prohíbe la aserción sobre datos no validados; esto lo refuerza donde
      // el verificador de fronteras no llega.
      "@typescript-eslint/no-unsafe-assignment": "warn",
      "@typescript-eslint/no-unsafe-member-access": "warn",

      /* --- Desactivadas porque svelte-check ya las cubre (§XIII) --- */
      "@typescript-eslint/no-explicit-any": "off",
      "svelte/valid-compile": "off",

      /* `AppError` es un objeto plano, no una subclase de `Error`, y eso es deliberado: cruza la
         frontera IPC serializado y una subclase de `Error` no sobrevive a `structuredClone`.
         El principio X fija su forma; lanzarlo es correcto aquí. */
      "@typescript-eslint/only-throw-error": "off",

      /* --- Estilo: lo decide prettier, no ESLint --- */
      "svelte/no-at-html-tags": "error" // salvo esta: renderizar HTML sin sanear está prohibido (§VI)
    }
  },

  {
    files: ["**/*.svelte"],
    languageOptions: {
      parser: svelteParser,
      parserOptions: { parser: ts.parser }
    },
    rules: {
      /* Las reglas de tipos producen 222 falsos positivos en ficheros `.svelte`: el parser no
         resuelve los tipos que el compilador de Svelte sí conoce. Ahí manda `svelte-check`, que
         usa el compilador real y ya se ejecuta con cero errores y cero avisos (§XIII). Mantenerlas
         aquí solo enseñaría a ignorar la salida de ESLint, que es peor que no tenerla. */
      "@typescript-eslint/no-unsafe-assignment": "off",
      "@typescript-eslint/no-unsafe-member-access": "off",
      "@typescript-eslint/no-unsafe-call": "off",
      "@typescript-eslint/no-unsafe-argument": "off",
      "@typescript-eslint/no-unsafe-return": "off"
    }
  },

  {
    // Los tests usan mocks y datos de prueba: exigirles el mismo rigor de tipos que al código de
    // producción produciría ruido sin valor.
    files: ["**/*.test.ts", "**/*.spec.ts"],
    rules: {
      "@typescript-eslint/no-unsafe-assignment": "off",
      "@typescript-eslint/no-unsafe-member-access": "off",
      "@typescript-eslint/no-unsafe-argument": "off",
      "@typescript-eslint/no-unsafe-call": "off",
      "@typescript-eslint/no-unsafe-return": "off"
    }
  },

  {
    // Scripts y herramientas: son Node, no la aplicación. `console` es su salida legítima.
    files: ["scripts/**/*.mjs", "tools/**/*.mjs", ".claude/hooks/**/*.mjs"],
    languageOptions: { globals: globals.node },
    // Fuera del `tsconfig` del proyecto: las reglas con información de tipos no pueden aplicarse.
    ...ts.configs.disableTypeChecked,
    rules: {
      ...ts.configs.disableTypeChecked.rules,
      "no-console": "off"
    }
  },

  {
    // Ficheros de configuración: viven fuera del `tsconfig` del proyecto, así que las reglas que
    // necesitan información de tipos no pueden aplicarse. Se comprueban sin ellas.
    files: ["*.config.js", "*.config.ts", "*.config.cjs", "svelte.config.js"],
    ...ts.configs.disableTypeChecked
  },

  {
    ignores: [
      "build/",
      ".svelte-kit/",
      "src-tauri/target/",
      "src-tauri/gen/",
      "coverage/",
      "node_modules/",
      "Design-system/",
      "historias.md"
    ]
  }
);
