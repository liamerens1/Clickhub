; ClickHub additions to Tauri's NSIS installer (bundle.windows.nsis.installerHooks).

; The uninstaller already deletes HKCU\...\Run\ClickHub when not updating. Also forget Task
; Manager's on/off record for it and the "autostart default applied" marker, so a later reinstall
; starts clean: on by default again, even when the user kept the application data.
!macro NSIS_HOOK_POSTUNINSTALL
  ${If} $UpdateMode <> 1
    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run" "ClickHub"
    SetShellVarContext current
    Delete "$LOCALAPPDATA\${BUNDLEID}\autostart-default-applied"
  ${EndIf}
!macroend
