# Empaqueta el entregable de rediseño en un único ZIP para enviar al diseñador.
#
#   pwsh design/entregable-rediseno/empaquetar.ps1
#
# Antes conviene regenerar las capturas y la galería:
#   $env:SDM_CAPTURAS = "1"; pnpm exec playwright test capturas
#   node design/entregable-rediseno/generar-indice.mjs

$ErrorActionPreference = "Stop"
$raiz = $PSScriptRoot
$destino = Join-Path $raiz "SmartDisk-entregable-rediseno.zip"

if (-not (Test-Path (Join-Path $raiz "salida/capturas"))) {
  throw "No hay capturas. Ejecuta primero: `$env:SDM_CAPTURAS='1'; pnpm exec playwright test capturas"
}

$incluir = @(
  Join-Path $raiz "README.md"
  Join-Path $raiz "COMO-ENTREGAR-EL-REDISENO.md"
  Join-Path $raiz "contexto"
  Join-Path $raiz "salida"
)

if (Test-Path $destino) { Remove-Item $destino -Force }
Compress-Archive -Path $incluir -DestinationPath $destino -CompressionLevel Optimal

$mb = [math]::Round((Get-Item $destino).Length / 1MB, 1)
Write-Host "Creado $destino ($mb MB)"
