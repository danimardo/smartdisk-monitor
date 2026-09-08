import { page } from "vitest/browser";
import { describe, expect, it } from "vitest";
import { render } from "vitest-browser-svelte";
import Markdown from "./Markdown.svelte";

/** `Markdown` pinta la respuesta de un LLM, que es **contenido no confiable** (principio XVI). Lo
 *  que jsdom no puede demostrar y un navegador sí: que el marcado activo colado en el texto queda
 *  como texto en el DOM real, sin crear nodos `<img>`, `<script>` ni enlaces navegables. */

describe("Markdown", () => {
  it("un encabezado ## se renderiza como elemento de título con su texto", async () => {
    await render(Markdown, { props: { source: "## Qué significa esto" } });
    await expect.element(page.getByRole("heading", { name: "Qué significa esto" })).toBeInTheDocument();
  });

  it("una lista '- ' produce elementos de lista reales", async () => {
    await render(Markdown, { props: { source: "- uno\n- dos" } });
    expect(document.querySelectorAll("li")).toHaveLength(2);
  });

  it("`código` en línea se renderiza en un <code>", async () => {
    await render(Markdown, { props: { source: "mira `Reallocated_Sector_Ct` aquí" } });
    await expect.element(page.getByText("Reallocated_Sector_Ct")).toBeInTheDocument();
    expect(document.querySelector("code")?.textContent).toBe("Reallocated_Sector_Ct");
  });

  it("una etiqueta <img onerror> sale como texto: no hay ningún <img> en el DOM", async () => {
    await render(Markdown, { props: { source: '<img src=x onerror="alert(1)"> y más texto' } });
    expect(document.querySelector("img")).toBeNull();
    await expect.element(page.getByText('<img src=x onerror="alert(1)"> y más texto')).toBeInTheDocument();
  });

  it("un <script> sale como texto, no se inserta ningún <script>", async () => {
    const cierre = "</" + "script>";
    await render(Markdown, { props: { source: `<script>fetch('/rob')${cierre}` } });
    // El único script posible sería el del propio bundle de test; el contenido no añade ninguno.
    expect([...document.querySelectorAll("script")].some((s) => s.textContent?.includes("/rob"))).toBe(false);
  });

  it("un enlace [texto](url) muestra el texto y la url, sin <a href>", async () => {
    await render(Markdown, { props: { source: "ver [la guía](https://malo.example/x)" } });
    expect(document.querySelector("a")).toBeNull();
    await expect.element(page.getByText("la guía (https://malo.example/x)")).toBeInTheDocument();
  });
});
