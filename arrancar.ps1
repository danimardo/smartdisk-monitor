# arrancar.ps1 - Detiene instancias previas/zombies y arranca SmartDisk Monitor en desarrollo
[CmdletBinding()]
param()

# Autoelevación: la aplicación necesita ejecutarse como administrador (ADR-004), y `cargo run`
# (lo que hace `pnpm app:dev` por debajo) nunca puede disparar el diálogo de UAC por sí solo —
# solo `Start-Process -Verb RunAs` (o hacer doble clic, o ya estar en una consola elevada) lo hace.
# Sin esto, la app se queda a medio arrancar con "requiere elevación" cada vez que este script se
# lanza desde una consola normal.
$esAdmin = ([Security.Principal.WindowsPrincipal] `
    [Security.Principal.WindowsIdentity]::GetCurrent()
    ).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $esAdmin) {
    Write-Host "Elevando privilegios..." -ForegroundColor Cyan
    Start-Process powershell.exe -Verb RunAs -WorkingDirectory $PSScriptRoot -ArgumentList @(
        "-NoExit", "-ExecutionPolicy", "Bypass", "-File", "`"$PSCommandPath`""
    )
    exit
}

$ErrorActionPreference = "Continue"

# `Start-Process -Verb RunAs` ignora `-WorkingDirectory` en la práctica: el proceso elevado nace en
# System32 pase lo que pase, algo ya conocido de cómo Windows arranca procesos elevados. Se fija el
# directorio aquí, dentro del propio script, en vez de confiar en el proceso que lo lanzó.
Set-Location -Path $PSScriptRoot

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  SmartDisk Monitor - Modo Desarrollo   " -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# 1. Comprobar y terminar procesos previos de la aplicación (smartdisk-monitor)
$processNames = @("smartdisk-monitor")
$runningProcesses = Get-Process -Name $processNames -ErrorAction SilentlyContinue

if ($runningProcesses) {
    Write-Host "[!] Se han detectado procesos previos o zombies de SmartDisk Monitor:" -ForegroundColor Yellow
    foreach ($proc in $runningProcesses) {
        Write-Host "    - PID: $($proc.Id) ($($proc.ProcessName))" -ForegroundColor Yellow
        try {
            Stop-Process -Id $proc.Id -Force -ErrorAction Stop
            Write-Host "    [OK] Proceso $($proc.Id) terminado." -ForegroundColor Green
        } catch {
            Write-Host "    [!] Reintentando con taskkill para PID $($proc.Id)..." -ForegroundColor DarkYellow
            taskkill /F /PID $proc.Id *>$null
        }
    }
    # Breve pausa para asegurar la liberación completa de recursos
    Start-Sleep -Milliseconds 600
} else {
    Write-Host "[OK] No hay instancias previas de smartdisk-monitor en ejecucion." -ForegroundColor Green
}

# 2. Comprobar si el puerto de desarrollo (1420) sigue ocupado por algun proceso residual
$port1420 = Get-NetTCPConnection -LocalPort 1420 -State Listen -ErrorAction SilentlyContinue
if ($port1420) {
    $pids = $port1420 | Select-Object -ExpandProperty OwningProcess -Unique
    foreach ($procId in $pids) {
        if ($procId -gt 0 -and $procId -ne $PID) {
            Write-Host "[!] Liberando puerto de desarrollo 1420 (PID $procId)..." -ForegroundColor Yellow
            try {
                Stop-Process -Id $procId -Force -ErrorAction SilentlyContinue
                Write-Host "    [OK] Proceso $procId terminado." -ForegroundColor Green
            } catch {
                taskkill /F /PID $procId *>$null
            }
        }
    }
    Start-Sleep -Milliseconds 400
}

# 3. Arrancar en modo desarrollo
Write-Host "`nIniciando 'pnpm app:dev'..." -ForegroundColor Cyan
pnpm app:dev
