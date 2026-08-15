!macro NSIS_HOOK_POSTINSTALL
  CopyFiles /SILENT "$INSTDIR\resources\*.dll" "$INSTDIR"
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  Delete "$INSTDIR\*.dll"
!macroend
