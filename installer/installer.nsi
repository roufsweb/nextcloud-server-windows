; ==============================================================================
; Nextcloud Server for Windows (All-In-One) - NSIS Production Installer
; Features: Modern UI, 64-bit architecture, Start Menu & Desktop shortcuts,
;           Windows Startup Run registration, and SAFE Uninstaller with data protection.
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

; Compression
SetCompressor /SOLID lzma
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

  ; Stop any existing running instances before copying
  DetailPrint "Checking for running instances..."
  nsExec::Exec 'taskkill /F /IM nextcloud-aio.exe /T'
  nsExec::Exec 'taskkill /F /IM nextcloud-aio-gui.exe /T'
  nsExec::Exec 'taskkill /F /IM caddy.exe /T'
  nsExec::Exec 'taskkill /F /IM php-cgi.exe /T'
  nsExec::Exec 'taskkill /F /IM cloudflared.exe /T'
  Sleep 1000

  ; Install everything from the staged portable distribution except user data and database
  File /r /x data /x "nextcloud.db*" /x "*.sqlite" "../dist/nextcloud-windows-aio/*"

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
  ; Terminate running processes before uninstalling
  DetailPrint "Terminating Nextcloud background processes..."
  nsExec::Exec 'taskkill /F /IM nextcloud-aio.exe /T'
  nsExec::Exec 'taskkill /F /IM nextcloud-aio-gui.exe /T'
  nsExec::Exec 'taskkill /F /IM caddy.exe /T'
  nsExec::Exec 'taskkill /F /IM php-cgi.exe /T'
  nsExec::Exec 'taskkill /F /IM cloudflared.exe /T'

  ; Check if user data directory or database exists
  StrCpy $PreserveUserData "1" ; Default to preserving user data
  IfFileExists "$INSTDIR\data\*.*" HasData CheckDb
CheckDb:
  IfFileExists "$INSTDIR\data\nextcloud.db" HasData NoData

HasData:
  MessageBox MB_YESNOCANCEL|MB_ICONQUESTION \
    "Nextcloud User Data & Database Protection:$\n$\nExisting user files and database were detected in:$\n$INSTDIR\data$\n$\nWould you like to KEEP your personal files and database?$\n$\n- Click YES to PRESERVE your personal data and database (Recommended).$\n- Click NO to permanently DELETE all personal data and database.$\n- Click CANCEL to abort uninstallation." \
    /SD IDYES IDYES KeepPersonalData IDNO DeletePersonalData
  Abort

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

  ; Remove Application Binaries, Runtimes, and UI Files
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

  ; Evaluate User Data Preservation
  ${If} $PreserveUserData == "1"
    DetailPrint "Preserving personal files and database in $INSTDIR\data"
    MessageBox MB_OK|MB_ICONINFORMATION \
      "Nextcloud application has been uninstalled successfully.$\n$\nYour personal files, documents, and database have been preserved safely at:$\n$INSTDIR\data"
  ${Else}
    DetailPrint "Removing personal data directory..."
    RMDir /r "$INSTDIR\data"
    RMDir /r "$INSTDIR\config"
    RMDir "$INSTDIR"
  ${EndIf}

SectionEnd
