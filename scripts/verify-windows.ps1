param(
    [Parameter(Mandatory = $true)]
    [string]$Binary,

    [Parameter(Mandatory = $true)]
    [ValidateSet("x64", "arm64")]
    [string]$Architecture
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path -LiteralPath $Binary -PathType Leaf)) {
    throw "Windows binary is missing: $Binary"
}

$dumpbinCommand = Get-Command dumpbin.exe -ErrorAction SilentlyContinue
if ($null -ne $dumpbinCommand) {
    $dumpbin = $dumpbinCommand.Source
} else {
    $vswhere = Join-Path ${env:ProgramFiles(x86)} "Microsoft Visual Studio/Installer/vswhere.exe"
    if (-not (Test-Path -LiteralPath $vswhere -PathType Leaf)) {
        throw "cannot locate dumpbin.exe or Visual Studio Installer's vswhere.exe"
    }
    $installationPath = (& $vswhere -latest -products * -property installationPath | Select-Object -First 1)
    if ([string]::IsNullOrWhiteSpace($installationPath)) {
        throw "vswhere.exe did not find a Visual Studio installation"
    }
    $dumpbin = Get-ChildItem -LiteralPath (Join-Path $installationPath "VC/Tools/MSVC") `
        -Filter dumpbin.exe -File -Recurse | Select-Object -First 1 -ExpandProperty FullName
    if ([string]::IsNullOrWhiteSpace($dumpbin)) {
        throw "Visual Studio does not contain dumpbin.exe"
    }
}

$headers = & $dumpbin /HEADERS $Binary | Out-String
if ($LASTEXITCODE -ne 0) {
    throw "dumpbin failed to inspect $Binary"
}

$expectedMachinePattern = if ($Architecture -eq "x64") { '8664\s+machine\s+\(x64\)' } else { 'AA64\s+machine\s+\(ARM64\)' }
if ($headers -notmatch $expectedMachinePattern) {
    throw "$Binary is not a Windows $Architecture executable"
}

$dependents = & $dumpbin /DEPENDENTS $Binary | Out-String
if ($LASTEXITCODE -ne 0) {
    throw "dumpbin failed to read the dependencies of $Binary"
}

# These libraries are supplied by the Visual C++ redistributable rather than
# by Windows itself. +crt-static must keep them out of release artifacts.
$redistributablePattern = '(?im)^\s*(vcruntime|msvcp|concrt|ucrtbased?|api-ms-win-crt)[^\s]*\.dll\s*$'
if ($dependents -match $redistributablePattern) {
    throw "$Binary dynamically links a Visual C++ runtime library: $($Matches[0].Trim())"
}

$dependencyPattern = '(?im)^\s*([A-Za-z0-9._-]+\.dll)\s*$'
$dependencyNames = @(
    [regex]::Matches($dependents, $dependencyPattern) |
        ForEach-Object { $_.Groups[1].Value } |
        Sort-Object -Unique
)
if ($dependencyNames.Count -eq 0) {
    throw "dumpbin did not report any DLL dependencies for $Binary"
}
foreach ($dependencyName in $dependencyNames) {
    $systemLibrary = Join-Path $env:SystemRoot "System32/$dependencyName"
    if (-not (Test-Path -LiteralPath $systemLibrary -PathType Leaf)) {
        throw "$Binary has a dynamic dependency that is not a Windows system DLL: $dependencyName"
    }
}

Write-Host "verified Windows $Architecture executable with only Windows system DLL dependencies: $Binary"
