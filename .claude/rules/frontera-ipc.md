---
paths:
  - "src/lib/api/**"
  - "src/lib/stores/**"
---

# Frontera con el backend

Es la única frontera real del proyecto: no hay HTTP, ni sesión, ni usuario remoto. Todo lo que
cruza entre TypeScript y Rust pasa por aquí, y ahí se concentra el riesgo de contrato.

- **Ninguna pantalla llama a `invoke` directamente.** Todo pasa por `$lib/api`, para que un cambio
  de firma rompa en un sitio y no en once.
- **Toda respuesta y todo evento se valida con su esquema Zod** antes de tocar el estado.
  `invoke<T>()` no valida nada: es una aserción de tipo sobre un dato que viene de otro proceso.
- **Los tipos se infieren con `z.infer<>`**, nunca se declaran a mano en paralelo. Dos
  declaraciones de la misma forma acaban divergiendo; una sola no puede.
- **Prohibida la aserción de tipo** (`as`) sobre datos que vengan de `invoke`, `listen`, ficheros
  o portapapeles. `as` sigue valiendo para estrechar algo ya validado.
- **Un fallo de esquema produce un `AppError`** con código `ipc.schema_mismatch` y la ruta del
  campo en el detalle técnico. Nunca se ignora ni se rellena con un valor por defecto: un dato con
  forma inesperada es un dato desconocido.
- **Los eventos traen el objeto completo**, no un parche: se reemplaza por identificador sin
  reconciliar.
- **Nada de sondeo.** El backend empuja (ADR-015); no hay `setInterval` pidiendo datos.

Todo esquema nuevo necesita su **prueba de rechazo**: uno probado solo con datos buenos no
demuestra nada. Lo que hay que verificar es que rechaza lo que debe rechazar.
