[Setup]
AppId={{8D6B6A79-4A3E-4C3B-9C9C-73A69E7F2A11}
AppName=CottonMD
AppVersion=0.2.0
AppPublisher=CottonMD
DefaultDirName={autopf}\CottonMD
DefaultGroupName=CottonMD
DisableProgramGroupPage=yes
OutputDir=..\dist
OutputBaseFilename=CottonMD-Setup-0.2.0-x64
SetupIconFile=..\assets\AppIcon.ico
UninstallDisplayIcon={app}\CottonMD.exe
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
ChangesAssociations=yes

[Tasks]
Name: "desktopicon"; Description: "Create a &desktop shortcut"; GroupDescription: "Additional icons:"; Flags: unchecked
Name: "addtopath"; Description: "Add cottonmd-cli to user PATH"; GroupDescription: "Command Line:"; Flags: unchecked

[Files]
Source: "..\target\release\CottonMD.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\target\release\cottonmd-cli.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\CottonMD"; Filename: "{app}\CottonMD.exe"
Name: "{autodesktop}\CottonMD"; Filename: "{app}\CottonMD.exe"; Tasks: desktopicon

[Registry]
Root: HKA; Subkey: "Software\Classes\.md\OpenWithProgids"; ValueType: string; ValueName: "CottonMD.Markdown"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\CottonMD.Markdown"; ValueType: string; ValueName: ""; ValueData: "Markdown Document"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\CottonMD.Markdown\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\CottonMD.exe,0"
Root: HKA; Subkey: "Software\Classes\CottonMD.Markdown\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\CottonMD.exe"" ""%1"""

[Run]
Filename: "{app}\CottonMD.exe"; Description: "Launch CottonMD"; Flags: nowait postinstall skipifsilent
