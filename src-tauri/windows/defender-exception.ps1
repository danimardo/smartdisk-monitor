# Añade o quita smartctl.exe de las aplicaciones permitidas de Control de acceso a carpetas de
# Windows Defender (J.56, ADR-043). Sin esta excepción, cualquier disco SATA que necesite ATA PASS
# THROUGH para SMART se bloquea en silencio, aunque el disco funcione perfectamente para todo lo
# demás (el Explorador de Windows nunca dispara esta protección).
#
# Nunca debe hacer fallar la instalación ni la desinstalación: si el cmdlet no existe en esta
# edición de Windows, o la Protección contra alteraciones bloquea el cambio incluso con privilegios
# de administrador, se sale con código 1 y quien invoque este script decide qué hacer (el
# instalador solo lo registra en su log; la aplicación ofrece reintentarlo desde su propia interfaz).

param(
    [Parameter(Mandatory = $true)]
    [ValidateSet("add", "remove")]
    [string]$Action,

    [Parameter(Mandatory = $true)]
    [string]$Path
)

try {
    if ($Action -eq "add") {
        Add-MpPreference -ControlledFolderAccessAllowedApplications $Path -ErrorAction Stop
    } else {
        Remove-MpPreference -ControlledFolderAccessAllowedApplications $Path -ErrorAction Stop
    }
    exit 0
} catch {
    exit 1
}
