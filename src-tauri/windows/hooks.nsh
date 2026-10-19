; ClickHub additions to Tauri's NSIS installer (bundle.windows.nsis.installerHooks).

; The uninstaller already deletes HKCU\...\Run\ClickHub when not updating. Also forget Task
; Manager's on/off record for it, so a later reinstall starts clean.
!macro NSIS_HOOK_POSTUNINSTALL
  ${If} $UpdateMode <> 1
    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run" "ClickHub"
  ${EndIf}
!macroend
