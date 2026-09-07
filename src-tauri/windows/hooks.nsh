; Hooks del instalador NSIS (J.56, ADR-043): smartctl.exe necesita estar en las aplicaciones
; permitidas de Control de acceso a carpetas de Windows Defender para poder leer SMART de un disco
; SATA — sin ello, el comando ATA PASS THROUGH se bloquea en silencio y el disco aparece siempre
; como "sin datos SMART" pese a funcionar perfectamente para todo lo demás.
;
; Ninguno de los dos hooks aborta la instalación/desinstalación si falla: puede que Windows
; Defender no esté activo, que la edición de Windows no tenga el cmdlet, o que la Protección contra
; alteraciones bloquee el cambio incluso con privilegios de administrador. En ese último caso, la
; propia aplicación ofrece reintentarlo desde su interfaz (comandos `check_smartctl_defender_exception`
; / `add_smartctl_defender_exception`, `src-tauri/src/commands/mod.rs`).

!macro NSIS_HOOK_POSTINSTALL
  DetailPrint "Permitiendo smartctl.exe en Control de acceso a carpetas de Windows Defender..."
  nsExec::ExecToLog '"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -NonInteractive -WindowStyle Hidden -ExecutionPolicy Bypass -File "$INSTDIR\scripts\defender-exception.ps1" -Action add -Path "$INSTDIR\bin\smartctl.exe"'
  Pop $0
  ${If} $0 != 0
    DetailPrint "No se pudo añadir la excepción (Protección contra alteraciones o cmdlet no disponible); la aplicación la reintentará desde su propia interfaz si hace falta."
  ${EndIf}
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  nsExec::ExecToLog '"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -NonInteractive -WindowStyle Hidden -ExecutionPolicy Bypass -File "$INSTDIR\scripts\defender-exception.ps1" -Action remove -Path "$INSTDIR\bin\smartctl.exe"'
  Pop $0
!macroend
