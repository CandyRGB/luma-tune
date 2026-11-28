Unicode true
RequestExecutionLevel admin
SetCompressor /SOLID lzma

!include "MUI2.nsh"
!include "FileFunc.nsh"
!include "LogicLib.nsh"
!include "WordFunc.nsh"
!include "x64.nsh"

!ifndef APP_EXE
    !define APP_EXE "..\target\x86_64-pc-windows-msvc\release\lumatune.exe"
!endif
!getdllversion "${APP_EXE}" APP_VER_
!define APP_VERSION "${APP_VER_1}.${APP_VER_2}.${APP_VER_3}.${APP_VER_4}"
!define UNINSTALL_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\LumaTune"

!ifndef SETUP_EXE
    !define SETUP_EXE "..\target\LumaTune-${APP_VERSION}-Setup-x64.exe"
!endif

Name "LumaTune ${APP_VERSION}"
OutFile "${SETUP_EXE}"
InstallDir "$PROGRAMFILES64\LumaTune"
VIProductVersion "${APP_VERSION}"
VIAddVersionKey /LANG=2052 "ProductName" "LumaTune"
VIAddVersionKey /LANG=2052 "FileDescription" "LumaTune 安装程序"
VIAddVersionKey /LANG=2052 "CompanyName" "CandyRGB"
VIAddVersionKey /LANG=2052 "LegalCopyright" "CandyRGB"
VIAddVersionKey /LANG=2052 "FileVersion" "${APP_VERSION}"
VIAddVersionKey /LANG=2052 "ProductVersion" "${APP_VERSION}"

!define MUI_ICON "..\apps\lumatune\ui\icons\lumatune.ico"
!define MUI_ABORTWARNING
!define MUI_PAGE_CUSTOMFUNCTION_PRE SkipDirectoryForInstalledApp
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "SimpChinese"

Var RegisteredDirectory
Var InstalledVersion
Var PendingExecutable

!macro LockSetup
    ; The process owns this handle until setup exits.
    System::Call 'kernel32::SetLastError(i 0)'
    System::Call 'kernel32::CreateMutexW(p 0, i 0, w "Global\CandyRGB.LumaTune.Setup") p.r0 ?e'
    Pop $1
    ${If} $0 == 0
    ${OrIf} $1 == 183
        MessageBox MB_ICONEXCLAMATION "另一个 LumaTune 安装或卸载程序正在运行。" /SD IDOK
        SetErrorLevel 1
        Abort
    ${EndIf}
!macroend

Function .onInit
    !insertmacro LockSetup
    ${IfNot} ${RunningX64}
        MessageBox MB_ICONSTOP "LumaTune 需要 64 位 Windows。" /SD IDOK
        SetErrorLevel 1
        Abort
    ${EndIf}
    SetRegView 64
    SetShellVarContext all
    ReadRegStr $RegisteredDirectory HKLM "${UNINSTALL_KEY}" "InstallLocation"
    ${If} $RegisteredDirectory == ""
        Return
    ${EndIf}

    StrCpy $INSTDIR $RegisteredDirectory
    ${GetFileVersion} "$INSTDIR\lumatune.exe" $InstalledVersion
    ${If} $InstalledVersion == ""
        MessageBox MB_YESNO|MB_ICONQUESTION "检测到已有安装，但无法读取程序版本。是否在原位置重新安装 ${APP_VERSION}？" /SD IDYES IDYES repair
        Abort
        repair:
        Return
    ${EndIf}

    ${VersionCompare} "${APP_VERSION}" "$InstalledVersion" $0
    ${If} $0 == 0
        MessageBox MB_OK|MB_ICONINFORMATION "本机已安装 ${APP_VERSION}，与此安装包版本一致，无需更新。" /SD IDOK
        SetErrorLevel 0
        Abort
    ${ElseIf} $0 == 2
        MessageBox MB_OK|MB_ICONINFORMATION "本机版本 $InstalledVersion 高于此安装包的 ${APP_VERSION}，无需更新。" /SD IDOK
        SetErrorLevel 0
        Abort
    ${EndIf}

    MessageBox MB_YESNO|MB_ICONQUESTION "本机版本为 $InstalledVersion。是否更新到 ${APP_VERSION}？安装位置：$INSTDIR" /SD IDYES IDYES update
    Abort
    update:
FunctionEnd

Function SkipDirectoryForInstalledApp
    ${If} $RegisteredDirectory != ""
        Abort
    ${EndIf}
FunctionEnd

Section "LumaTune"
    SetOutPath "$INSTDIR"
    ClearErrors
    GetTempFileName $PendingExecutable "$INSTDIR"
    IfErrors failed
    File /oname=$PendingExecutable "${APP_EXE}"
    IfErrors failed

    replace:
    ; Stage on the same volume so a failed replacement leaves the installed executable intact.
    System::Call 'kernel32::MoveFileExW(w "$PendingExecutable", w "$INSTDIR\lumatune.exe", i 9) i.r0'
    ${If} $0 == 0
        MessageBox MB_RETRYCANCEL|MB_ICONEXCLAMATION "无法替换 LumaTune。请先右击托盘图标退出程序，再重试。" /SD IDCANCEL IDRETRY replace
        Goto failed
    ${EndIf}

    ClearErrors
    WriteUninstaller "$INSTDIR\Uninstall.exe"
    WriteRegStr HKLM "${UNINSTALL_KEY}" "DisplayName" "LumaTune"
    WriteRegStr HKLM "${UNINSTALL_KEY}" "DisplayVersion" "${APP_VERSION}"
    WriteRegStr HKLM "${UNINSTALL_KEY}" "Publisher" "CandyRGB"
    WriteRegStr HKLM "${UNINSTALL_KEY}" "InstallLocation" "$INSTDIR"
    WriteRegStr HKLM "${UNINSTALL_KEY}" "DisplayIcon" "$INSTDIR\lumatune.exe"
    WriteRegStr HKLM "${UNINSTALL_KEY}" "UninstallString" '"$INSTDIR\Uninstall.exe"'
    WriteRegDWORD HKLM "${UNINSTALL_KEY}" "NoModify" 1
    WriteRegDWORD HKLM "${UNINSTALL_KEY}" "NoRepair" 1
    CreateShortcut "$SMPROGRAMS\LumaTune.lnk" "$INSTDIR\lumatune.exe"
    IfErrors failed
    Goto done

    failed:
    Delete "$PendingExecutable"
    SetErrorLevel 1
    Abort "LumaTune 安装未完成。"
    done:
SectionEnd

Function un.onInit
    !insertmacro LockSetup
    SetRegView 64
    SetShellVarContext all
FunctionEnd

Section "Uninstall"
    retry:
    ClearErrors
    Delete "$INSTDIR\lumatune.exe"
    ${If} ${Errors}
        MessageBox MB_RETRYCANCEL|MB_ICONEXCLAMATION "请先右击托盘图标退出程序，再重试卸载。" /SD IDCANCEL IDRETRY retry
        SetErrorLevel 1
        Abort
    ${EndIf}

    Delete "$SMPROGRAMS\LumaTune.lnk"
    DeleteRegKey HKLM "${UNINSTALL_KEY}"
    Delete "$INSTDIR\Uninstall.exe"
    SetOutPath "$TEMP"
    RMDir "$INSTDIR"
SectionEnd
