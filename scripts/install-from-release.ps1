# Install a forked grok release on Windows 11 (x64 or ARM64).
# Usage:
#   .\scripts\install-from-release.ps1
#   .\scripts\install-from-release.ps1 -Tag v0.2.110-plugin-hooks.1
#   .\scripts\install-from-release.ps1 -Tag v0.2.110-plugin-hooks.1 -Repo jonasvanderhaegen/grok-build
param(
    [string]$Tag = "",
    [string]$Repo = $(if ($env:GROK_RELEASE_REPO) { $env:GROK_RELEASE_REPO } else { "jonasvanderhaegen/grok-build" })
)

$ErrorActionPreference = "Stop"

$arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
switch ($arch) {
    "X64"   { $asset = "grok-windows-x86_64.zip"; break }
    "Arm64" { $asset = "grok-windows-aarch64.zip"; break }
    default { throw "Unsupported Windows architecture: $arch (need X64 or Arm64)" }
}

if (-not $Tag) {
    if (Get-Command gh -ErrorAction SilentlyContinue) {
        $Tag = gh release view --repo $Repo --json tagName -q .tagName
    } else {
        throw "Pass -Tag when gh is not installed (e.g. -Tag v0.2.110-plugin-hooks.1)"
    }
}
$Version = $Tag.TrimStart("v")
$DestDir = Join-Path $env:USERPROFILE ".grok\downloads"
$BinDir  = Join-Path $env:USERPROFILE ".grok\bin"
$Dest    = Join-Path $DestDir "grok-$Version-windows"
$Tmp     = Join-Path $env:TEMP ("grok-install-" + [guid]::NewGuid().ToString())
New-Item -ItemType Directory -Force -Path $Tmp, $DestDir, $BinDir | Out-Null

try {
    $url = "https://github.com/$Repo/releases/download/$Tag/$asset"
    Write-Host "Downloading $url"
    $zip = Join-Path $Tmp $asset
    if (Get-Command gh -ErrorAction SilentlyContinue) {
        gh release download $Tag --repo $Repo --pattern $asset --dir $Tmp
    } else {
        Invoke-WebRequest -Uri $url -OutFile $zip
    }
    Expand-Archive -Path (Join-Path $Tmp $asset) -DestinationPath $Tmp -Force
    $exe = Join-Path $Tmp "grok.exe"
    if (-not (Test-Path $exe)) { throw "Archive missing grok.exe" }

    Copy-Item $exe $Dest -Force
    Copy-Item $exe (Join-Path $BinDir "grok.exe") -Force
    Copy-Item $exe (Join-Path $BinDir "agent.exe") -Force

    # Ensure ~/.grok/bin is on user PATH
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($userPath -notlike "*$BinDir*") {
        [Environment]::SetEnvironmentVariable("Path", "$userPath;$BinDir", "User")
        $env:Path = "$env:Path;$BinDir"
        Write-Host "Added $BinDir to user PATH (open a new terminal to pick it up)."
    }

    Write-Host "Installed: $Dest"
    Write-Host "Also at:   $BinDir\grok.exe"
    & (Join-Path $BinDir "grok.exe") --version
    Write-Host "Open a new grok session so plugin hooks load at cold start."
}
finally {
    Remove-Item -Recurse -Force $Tmp -ErrorAction SilentlyContinue
}
