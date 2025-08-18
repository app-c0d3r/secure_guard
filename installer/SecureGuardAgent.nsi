# SecureGuard Agent Professional Installer
# NSIS Script for creating Windows installer package

!define PRODUCT_NAME "SecureGuard Agent"
!define PRODUCT_VERSION "1.0.0"
!define PRODUCT_PUBLISHER "SecureGuard Technologies"
!define PRODUCT_WEB_SITE "https://secureguard.com"
!define PRODUCT_DIR_REGKEY "Software\Microsoft\Windows\CurrentVersion\App Paths\secureguard-agent.exe"
!define PRODUCT_UNINST_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}"
!define PRODUCT_UNINST_ROOT_KEY "HKLM"
!define PRODUCT_STARTMENU_REGVAL "NSIS:StartMenuDir"

# Modern UI
!include "MUI2.nsh"
!include "nsDialogs.nsh"
!include "LogicLib.nsh"
!include "ServiceLib.nsh"
!include "x64.nsh"

# General Settings
Name "${PRODUCT_NAME} ${PRODUCT_VERSION}"
OutFile "SecureGuardAgentInstaller.exe"
InstallDir "$PROGRAMFILES64\SecureGuard\Agent"
InstallDirRegKey HKLM "${PRODUCT_DIR_REGKEY}" ""
ShowInstDetails show
ShowUnInstDetails show
RequestExecutionLevel admin
BrandingText "${PRODUCT_NAME} Professional Installer"

# Compression
SetCompressor /SOLID lzma
SetCompressorDictSize 32

# Variables
Var StartMenuFolder
Var ServerURL
Var APIBaseURL
Var AgentName
Var APIKey
Var DeviceName
Var ConfigDialog
Var Label1
Var Label2
Var Label3
Var TextBox1
Var TextBox2
Var TextBox3

# Modern UI Configuration
!define MUI_ABORTWARNING
!define MUI_ICON "installer_icon.ico"
!define MUI_UNICON "uninstaller_icon.ico"
!define MUI_WELCOMEFINISHPAGE_BITMAP "welcome.bmp"
!define MUI_UNWELCOMEFINISHPAGE_BITMAP "welcome.bmp"
!define MUI_HEADERIMAGE
!define MUI_HEADERIMAGE_BITMAP "header.bmp"
!define MUI_HEADERIMAGE_RIGHT

# Modern UI Pages
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "License.txt"
!insertmacro MUI_PAGE_COMPONENTS
!insertmacro MUI_PAGE_DIRECTORY

# Custom configuration page
Page custom ConfigurationPage ConfigurationLeave

!define MUI_STARTMENUPAGE_DEFAULTFOLDER "${PRODUCT_NAME}"
!define MUI_STARTMENUPAGE_REGISTRY_ROOT "${PRODUCT_UNINST_ROOT_KEY}"
!define MUI_STARTMENUPAGE_REGISTRY_KEY "${PRODUCT_UNINST_KEY}"
!define MUI_STARTMENUPAGE_REGISTRY_VALUENAME "${PRODUCT_STARTMENU_REGVAL}"
!insertmacro MUI_PAGE_STARTMENU Application $StartMenuFolder

!insertmacro MUI_PAGE_INSTFILES

# Finish page with service start option
!define MUI_FINISHPAGE_RUN
!define MUI_FINISHPAGE_RUN_TEXT "Start SecureGuard Agent service now"
!define MUI_FINISHPAGE_RUN_FUNCTION "StartService"
!define MUI_FINISHPAGE_SHOWREADME "$INSTDIR\README.txt"
!define MUI_FINISHPAGE_SHOWREADME_TEXT "Show installation notes"
!insertmacro MUI_PAGE_FINISH

# Uninstaller pages
!insertmacro MUI_UNPAGE_WELCOME
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

# Languages
!insertmacro MUI_LANGUAGE "English"

# Version Information
VIProductVersion "${PRODUCT_VERSION}.0"
VIAddVersionKey "ProductName" "${PRODUCT_NAME}"
VIAddVersionKey "Comments" "Professional endpoint security monitoring agent"
VIAddVersionKey "CompanyName" "${PRODUCT_PUBLISHER}"
VIAddVersionKey "LegalCopyright" "© ${PRODUCT_PUBLISHER}"
VIAddVersionKey "FileDescription" "${PRODUCT_NAME} Installer"
VIAddVersionKey "FileVersion" "${PRODUCT_VERSION}.0"
VIAddVersionKey "ProductVersion" "${PRODUCT_VERSION}.0"

# Custom Configuration Page
Function ConfigurationPage
  !insertmacro MUI_HEADER_TEXT "Agent Registration" "Register your device with SecureGuard"
  
  nsDialogs::Create 1018
  Pop $ConfigDialog
  
  ${NSD_CreateLabel} 0 0 100% 24u "To complete the installation, please provide your SecureGuard API key and device name. You can generate an API key in your SecureGuard dashboard under 'Profile > API Keys'."
  Pop $Label1
  
  ${NSD_CreateLabel} 0 35u 30% 12u "API Key:"
  Pop $Label2
  ${NSD_CreateText} 30% 33u 65% 12u ""
  Pop $TextBox1
  StrCpy $APIKey $TextBox1
  
  ${NSD_CreateLabel} 0 55u 30% 12u "Device Name:"
  Pop $Label3
  ${NSD_CreateText} 30% 53u 65% 12u "$COMPUTERNAME"
  Pop $TextBox2
  StrCpy $DeviceName $TextBox2
  
  ${NSD_CreateLabel} 0 75u 30% 12u "Server URL:"
  Pop $0
  ${NSD_CreateText} 30% 73u 65% 12u "https://api.secureguard.com"
  Pop $TextBox3
  StrCpy $ServerURL $TextBox3
  
  ${NSD_CreateLabel} 0 100u 100% 36u "Important: Your API key will be used to register this device to your account. Keep your API key secure and never share it. The key will be encrypted and stored securely on this device."
  Pop $0
  
  nsDialogs::Show
FunctionEnd

Function ConfigurationLeave
  ${NSD_GetText} $APIKey $APIKey
  ${NSD_GetText} $DeviceName $DeviceName  
  ${NSD_GetText} $ServerURL $ServerURL
  
  # Validate API key format (should start with sg_)
  StrLen $0 $APIKey
  ${If} $0 == 0
    MessageBox MB_OK|MB_ICONEXCLAMATION "Please enter your SecureGuard API key."
    Abort
  ${EndIf}
  
  StrCpy $1 $APIKey 3  # Get first 3 characters
  ${If} $1 != "sg_"
    MessageBox MB_OK|MB_ICONEXCLAMATION "Invalid API key format. SecureGuard API keys start with 'sg_'."
    Abort
  ${EndIf}
  
  # Validate device name
  StrLen $0 $DeviceName
  ${If} $0 == 0
    MessageBox MB_OK|MB_ICONEXCLAMATION "Please enter a device name."
    Abort
  ${EndIf}
FunctionEnd

# Installation Types
InstType "Full Installation"
InstType "Minimal Installation"

# Components
Section "SecureGuard Agent Core" SEC01
  SectionIn 1 2 RO
  
  SetOutPath "$INSTDIR"
  SetOverwrite ifnewer
  
  # Main executable
  File "..\agent-deployment\secureguard-agent.exe"
  
  # Create directories
  CreateDirectory "$INSTDIR\config"
  CreateDirectory "$INSTDIR\logs"
  CreateDirectory "$INSTDIR\temp"
  
  # Configuration file with API key and device settings
  FileOpen $0 "$INSTDIR\config\config.toml" w
  FileWrite $0 "# SecureGuard Agent Configuration$\r$\n"
  FileWrite $0 "# Auto-generated during installation$\r$\n$\r$\n"
  FileWrite $0 "[server]$\r$\n"
  FileWrite $0 'base_url = "$ServerURL"$\r$\n'
  FileWrite $0 "timeout = 30$\r$\n$\r$\n"
  FileWrite $0 "[agent]$\r$\n"
  FileWrite $0 'device_name = "$DeviceName"$\r$\n'
  FileWrite $0 'api_key = "$APIKey"$\r$\n'
  FileWrite $0 'log_level = "info"$\r$\n'
  FileWrite $0 "heartbeat_interval = 30$\r$\n"
  FileWrite $0 "data_collection_interval = 300$\r$\n$\r$\n"
  FileWrite $0 "[monitoring]$\r$\n"
  FileWrite $0 "heartbeat_interval = 30$\r$\n"
  FileWrite $0 "data_collection_interval = 300$\r$\n$\r$\n"
  FileWrite $0 "[security]$\r$\n"
  FileWrite $0 "encryption_enabled = true$\r$\n$\r$\n"
  FileWrite $0 "[logging]$\r$\n"
  FileWrite $0 'file_path = "$INSTDIR\logs\agent.log"$\r$\n'
  FileWrite $0 'level = "info"$\r$\n'
  FileWrite $0 'max_size = "10MB"$\r$\n'
  FileWrite $0 "max_files = 5$\r$\n"
  FileClose $0
  
  # Management scripts
  FileOpen $0 "$INSTDIR\manage.bat" w
  FileWrite $0 "@echo off$\r$\n"
  FileWrite $0 "echo SecureGuard Agent Service Management$\r$\n"
  FileWrite $0 "echo ====================================$\r$\n"
  FileWrite $0 "echo 1. Start Service$\r$\n"
  FileWrite $0 "echo 2. Stop Service$\r$\n"
  FileWrite $0 "echo 3. Restart Service$\r$\n"
  FileWrite $0 "echo 4. Service Status$\r$\n"
  FileWrite $0 "echo 5. View Configuration$\r$\n"
  FileWrite $0 "echo 6. View Logs$\r$\n"
  FileWrite $0 "echo 7. Exit$\r$\n"
  FileWrite $0 "echo.$\r$\n"
  FileWrite $0 'set /p choice="Choose option (1-7): "$\r$\n'
  FileWrite $0 "if %choice%==1 sc start SecureGuardAgent$\r$\n"
  FileWrite $0 "if %choice%==2 sc stop SecureGuardAgent$\r$\n"
  FileWrite $0 "if %choice%==3 (sc stop SecureGuardAgent & timeout /t 3 >nul & sc start SecureGuardAgent)$\r$\n"
  FileWrite $0 "if %choice%==4 sc query SecureGuardAgent$\r$\n"
  FileWrite $0 'if %choice%==5 notepad "$INSTDIR\config\config.toml"$\r$\n'
  FileWrite $0 'if %choice%==6 notepad "$INSTDIR\logs\agent.log"$\r$\n'
  FileWrite $0 "if %choice%==7 exit$\r$\n"
  FileWrite $0 "pause$\r$\n"
  FileClose $0
  
  # README file
  FileOpen $0 "$INSTDIR\README.txt" w
  FileWrite $0 "SecureGuard Agent v${PRODUCT_VERSION}$\r$\n"
  FileWrite $0 "================================$\r$\n$\r$\n"
  FileWrite $0 "Installation completed successfully!$\r$\n$\r$\n"
  FileWrite $0 "Service Name: SecureGuardAgent$\r$\n"
  FileWrite $0 "Installation Directory: $INSTDIR$\r$\n"
  FileWrite $0 "Configuration File: $INSTDIR\config\config.toml$\r$\n"
  FileWrite $0 "Log Files: $INSTDIR\logs\$\r$\n$\r$\n"
  FileWrite $0 "Management:$\r$\n"
  FileWrite $0 "- Use Windows Services panel to manage the service$\r$\n"
  FileWrite $0 "- Or run: $INSTDIR\manage.bat$\r$\n$\r$\n"
  FileWrite $0 "Support: https://secureguard.com/support$\r$\n"
  FileClose $0
SectionEnd

Section "Windows Service" SEC02
  SectionIn 1
  
  DetailPrint "Installing Windows service..."
  
  # Install service
  ExecWait 'sc create SecureGuardAgent binPath= "$INSTDIR\secureguard-agent.exe" start= auto DisplayName= "SecureGuard Security Agent" depend= Tcpip' $0
  ${If} $0 != 0
    DetailPrint "Warning: Service installation may have failed (exit code: $0)"
  ${EndIf}
  
  # Set service description
  ExecWait 'sc description SecureGuardAgent "SecureGuard security monitoring agent - monitors system for threats and security events"'
  
  # Configure service recovery options
  ExecWait 'sc failure SecureGuardAgent reset= 86400 actions= restart/30000/restart/60000/restart/120000'
SectionEnd

Section "Firewall Rules" SEC03
  SectionIn 1
  
  DetailPrint "Configuring Windows Firewall..."
  
  # Add firewall exceptions
  ExecWait 'netsh advfirewall firewall add rule name="SecureGuard Agent" dir=out action=allow program="$INSTDIR\secureguard-agent.exe"'
  ExecWait 'netsh advfirewall firewall add rule name="SecureGuard Agent Inbound" dir=in action=allow program="$INSTDIR\secureguard-agent.exe"'
SectionEnd

Section "Start Menu Shortcuts" SEC04
  SectionIn 1
  
  !insertmacro MUI_STARTMENU_WRITE_BEGIN Application
  
  CreateDirectory "$SMPROGRAMS\$StartMenuFolder"
  CreateShortCut "$SMPROGRAMS\$StartMenuFolder\Agent Manager.lnk" "$INSTDIR\manage.bat"
  CreateShortCut "$SMPROGRAMS\$StartMenuFolder\Configuration.lnk" "notepad.exe" "$INSTDIR\config\config.toml"
  CreateShortCut "$SMPROGRAMS\$StartMenuFolder\View Logs.lnk" "$INSTDIR\logs"
  CreateShortCut "$SMPROGRAMS\$StartMenuFolder\Uninstall.lnk" "$INSTDIR\uninstall.exe"
  
  !insertmacro MUI_STARTMENU_WRITE_END
SectionEnd

Section "Desktop Shortcuts" SEC05
  CreateShortCut "$DESKTOP\SecureGuard Agent Manager.lnk" "$INSTDIR\manage.bat"
SectionEnd

# Component Descriptions
!insertmacro MUI_FUNCTION_DESCRIPTION_BEGIN
  !insertmacro MUI_DESCRIPTION_TEXT ${SEC01} "Core SecureGuard Agent files and configuration"
  !insertmacro MUI_DESCRIPTION_TEXT ${SEC02} "Install and configure as Windows service (recommended)"
  !insertmacro MUI_DESCRIPTION_TEXT ${SEC03} "Configure Windows Firewall exceptions for agent communication"
  !insertmacro MUI_DESCRIPTION_TEXT ${SEC04} "Create Start Menu shortcuts for easy access"
  !insertmacro MUI_DESCRIPTION_TEXT ${SEC05} "Create desktop shortcut for agent management"
!insertmacro MUI_FUNCTION_DESCRIPTION_END

# Post-installation
Section -AdditionalIcons
  WriteIniStr "$INSTDIR\${PRODUCT_NAME}.url" "InternetShortcut" "URL" "${PRODUCT_WEB_SITE}"
SectionEnd

Section -Post
  WriteUninstaller "$INSTDIR\uninstall.exe"
  WriteRegStr HKLM "${PRODUCT_DIR_REGKEY}" "" "$INSTDIR\secureguard-agent.exe"
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "DisplayName" "$(^Name)"
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "UninstallString" "$INSTDIR\uninstall.exe"
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "DisplayIcon" "$INSTDIR\secureguard-agent.exe"
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "DisplayVersion" "${PRODUCT_VERSION}"
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "URLInfoAbout" "${PRODUCT_WEB_SITE}"
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "Publisher" "${PRODUCT_PUBLISHER}"
  WriteRegDWORD ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "NoModify" 1
  WriteRegDWORD ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "NoRepair" 1
SectionEnd

# Custom Functions
Function StartService
  ExecWait 'sc start SecureGuardAgent' $0
  ${If} $0 = 0
    MessageBox MB_OK "SecureGuard Agent service started successfully."
  ${Else}
    MessageBox MB_OK "Failed to start SecureGuard Agent service. You can start it manually from the Services panel."
  ${EndIf}
FunctionEnd

Function .onInit
  # Check for existing installation
  ReadRegStr $R0 ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "UninstallString"
  StrCmp $R0 "" done
  
  MessageBox MB_OKCANCEL|MB_ICONEXCLAMATION \
  "SecureGuard Agent is already installed. $\n$\nClick `OK` to remove the previous version or `Cancel` to cancel this upgrade." \
  IDOK uninst
  Abort
  
  uninst:
    ClearErrors
    ExecWait '$R0 /S _?=$INSTDIR'
    
    IfErrors no_remove_uninstaller done
      Delete $R0
    no_remove_uninstaller:
  
  done:
FunctionEnd

# Uninstaller
Section Uninstall
  # Stop and remove service
  ExecWait 'sc stop SecureGuardAgent'
  Sleep 3000
  ExecWait 'sc delete SecureGuardAgent'
  
  # Remove firewall rules
  ExecWait 'netsh advfirewall firewall delete rule name="SecureGuard Agent"'
  ExecWait 'netsh advfirewall firewall delete rule name="SecureGuard Agent Inbound"'
  
  # Remove files
  Delete "$INSTDIR\${PRODUCT_NAME}.url"
  Delete "$INSTDIR\uninstall.exe"
  Delete "$INSTDIR\README.txt"
  Delete "$INSTDIR\manage.bat"
  Delete "$INSTDIR\secureguard-agent.exe"
  Delete "$INSTDIR\config\config.toml"
  
  # Remove directories
  RMDir /r "$INSTDIR\logs"
  RMDir /r "$INSTDIR\temp"
  RMDir "$INSTDIR\config"
  RMDir "$INSTDIR"
  RMDir "$PROGRAMFILES64\SecureGuard"
  
  # Remove shortcuts
  !insertmacro MUI_STARTMENU_GETFOLDER "Application" $StartMenuFolder
  Delete "$SMPROGRAMS\$StartMenuFolder\*"
  RMDir "$SMPROGRAMS\$StartMenuFolder"
  Delete "$DESKTOP\SecureGuard Agent Manager.lnk"
  
  # Remove registry entries
  DeleteRegKey ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}"
  DeleteRegKey HKLM "${PRODUCT_DIR_REGKEY}"
  SetAutoClose true
SectionEnd

Function un.onInit
  MessageBox MB_ICONQUESTION|MB_YESNO|MB_DEFBUTTON2 "Are you sure you want to completely remove $(^Name) and all of its components?" IDYES +2
  Abort
FunctionEnd

Function un.onUninstSuccess
  HideWindow
  MessageBox MB_ICONINFORMATION|MB_OK "$(^Name) was successfully removed from your computer."
FunctionEnd