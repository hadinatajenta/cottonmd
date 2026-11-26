$ErrorActionPreference = "Stop"

$RootDir = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$DistDir = Join-Path $RootDir "dist\CottonMD-windows-x64"
$ZipPath = Join-Path $RootDir "dist\CottonMD-windows-x64.zip"

Push-Location $RootDir
try {
    cargo build --release -p cottonmd-desktop -p cottonmd-cli

    if (Test-Path $DistDir) { Remove-Item -Recurse -Force $DistDir }
    if (Test-Path $ZipPath) { Remove-Item -Force $ZipPath }
    New-Item -ItemType Directory -Path $DistDir -Force | Out-Null

    Copy-Item "target\release\CottonMD.exe" (Join-Path $DistDir "CottonMD.exe")
    Copy-Item "target\release\cottonmd-cli.exe" (Join-Path $DistDir "cottonmd-cli.exe")
    Copy-Item "README.md" (Join-Path $DistDir "README.md")

    Compress-Archive -Path "$DistDir\*" -DestinationPath $ZipPath -Force
    Write-Host "Packaged Windows portable archive: $ZipPath"
    Get-Item $ZipPath, (Join-Path $DistDir "CottonMD.exe"), (Join-Path $DistDir "cottonmd-cli.exe") | Format-Table Name, Length
}
finally {
    Pop-Location
}
