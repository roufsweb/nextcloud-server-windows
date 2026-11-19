; ==============================================================================
; Nextcloud Server for Windows (All-In-One) - NSIS Production Installer
; Features: Modern UI, 64-bit architecture, High-Speed extraction & uninstallation,
;           Start Menu & Desktop shortcuts, and SAFE Uninstaller with data protection.
; ==============================================================================

!include "MUI2.nsh"
!include "LogicLib.nsh"
!include "FileFunc.nsh"
!include "x64.nsh"

; General Definitions
Name "Nextcloud Server for Windows"
OutFile "../dist/nextcloud-windows-aio-v1.0.0-setup-x64.exe"
InstallDir "$LOCALAPPDATA\Programs\Nextcloud Server Windows"
InstallDirRegKey HKCU "Software\NextcloudServerWindows" "InstallDir"
RequestExecutionLevel user ; Clean installation in user space without requiring UAC root

; Compression Configuration (Optimized for speed and high-ratio decompression)
SetCompressor /SOLID lzma
SetCompressorDictSize 16
Unicode True

; Interface Settings
!define MUI_ABORTWARNING
!define MUI_ICON "../docs/img/logo.ico"
!define MUI_UNICON "../docs/img/logo.ico"

; Welcome page
!insertmacro MUI_PAGE_WELCOME
; License page
!insertmacro MUI_PAGE_LICENSE "../LICENSE"
; Directory selection page
!insertmacro MUI_PAGE_DIRECTORY
; Instfiles page
!insertmacro MUI_PAGE_INSTFILES
; Finish page
!define MUI_FINISHPAGE_RUN "$INSTDIR\nextcloud-aio-gui.exe"
!define MUI_FINISHPAGE_RUN_TEXT "Launch Nextcloud Server for Windows"
!insertmacro MUI_PAGE_FINISH

; Uninstaller pages
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

; Language
!insertmacro MUI_LANGUAGE "English"

; Version Information
VIProductVersion "1.0.0.0"
VIAddVersionKey "ProductName" "Nextcloud Server for Windows"
VIAddVersionKey "CompanyName" "roufsweb"
VIAddVersionKey "LegalCopyright" "Copyright (C) 2026 roufsweb. GNU AGPLv3."
VIAddVersionKey "FileDescription" "Nextcloud Server for Windows Setup"
VIAddVersionKey "FileVersion" "1.0.0.0"
VIAddVersionKey "ProductVersion" "1.0.0.0"

; ------------------------------------------------------------------------------
; Installer Section
; ------------------------------------------------------------------------------
Section "Nextcloud Server Core Stack" SecCore
  SetOutPath "$INSTDIR"

  ; Swiftly terminate any running instances in a single fast command
  DetailPrint "Checking for running Nextcloud instances..."
  nsExec::Exec 'cmd /c taskkill /F /T /IM nextcloud-aio*.exe /IM caddy.exe /IM php-cgi.exe /IM cloudflared.exe >nul 2>&1'

  ; High-speed extraction: suppress Win32 ListBox GDI redraws for 25,000+ files
  SetDetailsPrint textonly
  DetailPrint "Extracting Nextcloud core application files (high-speed unpack)..."
  SetDetailsPrint none

  ; Install everything from the staged portable distribution except user data and database
  File /r /x data /x "nextcloud.db*" /x "*.sqlite" "../dist/nextcloud-windows-aio/*"

  SetDetailsPrint both
  DetailPrint "Configuring Windows application environment and shortcuts..."

  ; Create default empty data directory if not exists
  IfFileExists "$INSTDIR\data\*.*" DataExists CreateData
CreateData:
  CreateDirectory "$INSTDIR\data"
DataExists:

  ; Write Uninstaller
  SetOutPath "$INSTDIR"
  WriteUninstaller "$INSTDIR\uninstall.exe"

  ; Store Installation Path
  WriteRegStr HKCU "Software\NextcloudServerWindows" "InstallDir" "$INSTDIR"

  ; Windows Add/Remove Programs Registration
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\NextcloudServerWindows" "DisplayName" "Nextcloud Server for Windows (All-In-One)"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\NextcloudServerWindows" "DisplayVersion" "1.0.0"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\NextcloudServerWindows" "Publisher" "roufsweb"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\NextcloudServerWindows" "UninstallString" '"$INSTDIR\uninstall.exe"'
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\NextcloudServerWindows" "QuietUninstallString" '"$INSTDIR\uninstall.exe" /S'
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\NextcloudServerWindows" "InstallLocation" "$INSTDIR"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\NextcloudServerWindows" "DisplayIcon" '"$INSTDIR\nextcloud-aio-gui.exe"'
  WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\NextcloudServerWindows" "NoModify" 1
  WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\NextcloudServerWindows" "NoRepair" 1

  ; Create Start Menu Shortcuts
  CreateDirectory "$SMPROGRAMS\Nextcloud Server for Windows"
  CreateShortcut "$SMPROGRAMS\Nextcloud Server for Windows\Nextcloud Server.lnk" "$INSTDIR\nextcloud-aio-gui.exe" "" "$INSTDIR\nextcloud-aio-gui.exe" 0
  CreateShortcut "$SMPROGRAMS\Nextcloud Server for Windows\Nextcloud Web Dashboard.lnk" "http://localhost:8080" "" "" 0
  CreateShortcut "$SMPROGRAMS\Nextcloud Server for Windows\Uninstall Nextcloud Server.lnk" "$INSTDIR\uninstall.exe" "" "$INSTDIR\uninstall.exe" 0

  ; Create Desktop Shortcut
  CreateShortcut "$DESKTOP\Nextcloud Server for Windows.lnk" "$INSTDIR\nextcloud-aio-gui.exe" "" "$INSTDIR\nextcloud-aio-gui.exe" 0

SectionEnd

; ------------------------------------------------------------------------------
; Safe Uninstaller Section
; ------------------------------------------------------------------------------
Var PreserveUserData

Function un.onInit
  ; Swiftly terminate any running instances in a single fast command
  nsExec::Exec 'cmd /c taskkill /F /T /IM nextcloud-aio*.exe /IM caddy.exe /IM php-cgi.exe /IM cloudflared.exe >nul 2>&1'

  ; Default to preserving personal data
  StrCpy $PreserveUserData "1"

  ; Check if user data directory or database exists
  IfFileExists "$INSTDIR\data\*.*" HasData CheckDb
CheckDb:
  IfFileExists "$INSTDIR\data\nextcloud.db" HasData NoData

HasData:
  MessageBox MB_YESNO|MB_ICONQUESTION \
    "Nextcloud User Data & Database Protection:$\n$\nExisting user storage and database were detected in:$\n$INSTDIR\data$\n$\nWould you like to KEEP your personal files and database?$\n$\n- Click YES to PRESERVE your personal data and database (Recommended).$\n- Click NO to permanently DELETE all personal data and database." \
    /SD IDYES IDYES KeepPersonalData IDNO DeletePersonalData

KeepPersonalData:
  StrCpy $PreserveUserData "1"
  Goto EndDataCheck

DeletePersonalData:
  StrCpy $PreserveUserData "0"
  Goto EndDataCheck

NoData:
  StrCpy $PreserveUserData "0"

EndDataCheck:
FunctionEnd

Section "Uninstall"
  ; Remove Windows Startup Run key if configured
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "NextcloudAIO"

  ; Remove Add/Remove Programs registration
  DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\NextcloudServerWindows"
  DeleteRegKey HKCU "Software\NextcloudServerWindows"

  ; Remove Start Menu & Desktop Shortcuts
  Delete "$DESKTOP\Nextcloud Server for Windows.lnk"
  Delete "$SMPROGRAMS\Nextcloud Server for Windows\Nextcloud Server.lnk"
  Delete "$SMPROGRAMS\Nextcloud Server for Windows\Nextcloud Web Dashboard.lnk"
  Delete "$SMPROGRAMS\Nextcloud Server for Windows\Uninstall Nextcloud Server.lnk"
  RMDir "$SMPROGRAMS\Nextcloud Server for Windows"

  ; High-speed deletion: suppress Win32 ListBox GDI redraws for thousands of deleted files
  SetDetailsPrint textonly
  DetailPrint "Removing Nextcloud runtime binaries and application stack..."
  SetDetailsPrint none

  Delete "$INSTDIR\nextcloud-aio.exe"
  Delete "$INSTDIR\nextcloud-aio-gui.exe"
  Delete "$INSTDIR\Nextcloud-AIO.vbs"
  Delete "$INSTDIR\start-server.bat"
  Delete "$INSTDIR\open-dashboard.bat"
  Delete "$INSTDIR\LICENSE"
  Delete "$INSTDIR\README.md"
  Delete "$INSTDIR\THIRD_PARTY_LICENSES.md"
  Delete "$INSTDIR\uninstall.exe"

  RMDir /r "$INSTDIR\caddy"
  RMDir /r "$INSTDIR\php"
  RMDir /r "$INSTDIR\cloudflared"
  RMDir /r "$INSTDIR\runtime"
  RMDir /r "$INSTDIR\ui"
  RMDir /r "$INSTDIR\nextcloud"
  RMDir /r "$INSTDIR\logs"
  RMDir /r "$INSTDIR\tmp"

  ; Evaluate User Data Preservation (without jarring blocking popups)
  ${If} $PreserveUserData == "1"
    SetDetailsPrint both
    DetailPrint "Personal files and database preserved in: $INSTDIR\data"
    SetDetailsPrint none
  ${Else}
    RMDir /r "$INSTDIR\data"
    RMDir /r "$INSTDIR\config"
    RMDir "$INSTDIR"
  ${EndIf}

  SetDetailsPrint both
  DetailPrint "Uninstallation complete."

SectionEnd
