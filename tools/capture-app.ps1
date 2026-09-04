# Arranca la aplicación compilada, espera a que aparezca su ventana y la captura.
# Sirve para comprobar de un vistazo que el esqueleto pinta lo que debe pintar.
#
#   powershell -NoProfile -ExecutionPolicy Bypass -File tools\capture-app.ps1 [-Seconds 8]

param(
  [string]$Exe = "$PSScriptRoot\..\src-tauri\target\debug\smartdisk-monitor.exe",
  [string]$Out = "$PSScriptRoot\..\app-arranque.png",
  [int]$Seconds = 8
)

Add-Type -AssemblyName System.Drawing
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class ScreenCap {
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  // PrintWindow captura el contenido real de la ventana aunque esté tapada por otra.
  // CopyFromScreen captura la PANTALLA en esas coordenadas, que no es lo mismo.
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr hdc, uint flags);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowTextW(IntPtr h, System.Text.StringBuilder s, int n);
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc cb, IntPtr p);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  public delegate bool EnumProc(IntPtr h, IntPtr p);
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L; public int T; public int R; public int B; }

  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetClassNameW(IntPtr h, System.Text.StringBuilder s, int n);

  // Un proceso Tauri tiene varias ventanas (Tao Thread Event Target, IME, PseudoConsole...).
  // La real es la de clase "Tauri Window"; el área sola no la distingue de forma fiable.
  public static IntPtr MainWindowOf(uint targetPid) {
    IntPtr best = IntPtr.Zero; long bestArea = 0;
    EnumWindows(delegate(IntPtr h, IntPtr p) {
      uint pid; GetWindowThreadProcessId(h, out pid);
      if (pid != targetPid || !IsWindowVisible(h)) return true;
      var cls = new System.Text.StringBuilder(256);
      GetClassNameW(h, cls, 256);
      if (cls.ToString() != "Tauri Window") return true;
      RECT r; if (!GetWindowRect(h, out r)) return true;
      long area = (long)(r.R - r.L) * (r.B - r.T);
      if (area > bestArea) { bestArea = area; best = h; }
      return true;
    }, IntPtr.Zero);
    return best;
  }
}
"@

$proc = Start-Process -FilePath $Exe -PassThru
Write-Output "PID $($proc.Id), esperando la ventana..."

# Se busca por el proceso, no por el título: el título puede tardar en fijarse.
$handle = [IntPtr]::Zero
for ($i = 0; $i -lt ($Seconds * 4) -and $handle -eq [IntPtr]::Zero; $i++) {
  Start-Sleep -Milliseconds 250
  $live = Get-Process -Id $proc.Id -ErrorAction SilentlyContinue
  if ($null -eq $live) { Write-Output "El proceso terminó por su cuenta"; exit 1 }
  $live.Refresh()
  $candidate = [ScreenCap]::MainWindowOf([uint32]$proc.Id)
  if ($candidate -ne [IntPtr]::Zero) { $handle = $candidate }
}

if ($handle -eq [IntPtr]::Zero) {
  Write-Output "No apareció ninguna ventana en $Seconds s"
  $proc | Stop-Process -Force
  exit 1
}

$sb = New-Object System.Text.StringBuilder 256
[ScreenCap]::GetWindowTextW($handle, $sb, 256) | Out-Null
Write-Output "Ventana: '$($sb.ToString())'"
[ScreenCap]::SetForegroundWindow($handle) | Out-Null
Start-Sleep -Seconds 2

$r = New-Object ScreenCap+RECT
[ScreenCap]::GetWindowRect($handle, [ref]$r) | Out-Null
$w = $r.R - $r.L
$h = $r.B - $r.T
Write-Output "Tamaño: ${w}x${h}"

$bmp = New-Object System.Drawing.Bitmap $w, $h
$g = [System.Drawing.Graphics]::FromImage($bmp)
$hdc = $g.GetHdc()
# PW_RENDERFULLCONTENT (0x2): necesario para ventanas con contenido acelerado como WebView2.
$printed = [ScreenCap]::PrintWindow($handle, $hdc, 2)
$g.ReleaseHdc($hdc)
if (-not $printed) {
  Write-Output "PrintWindow falló; se recurre a capturar la pantalla"
  $g.CopyFromScreen($r.L, $r.T, 0, 0, $bmp.Size)
}
$bmp.Save($Out, [System.Drawing.Imaging.ImageFormat]::Png)
$g.Dispose()
$bmp.Dispose()
Write-Output "Captura: $Out"

$proc | Stop-Process -Force
Write-Output "Proceso detenido"
