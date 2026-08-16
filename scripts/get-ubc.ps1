#Requires -Version 5.1

<#
.SYNOPSIS
    Download the pinned `ubc` (ubCode) binary into tools/.

.DESCRIPTION
    Agconflo's documentation toolchain is `ubc` and nothing else - no Python, no
    Sphinx, no Java. This script is the whole installation step.

    The binary is ~66 MB and is NEVER committed; tools/ is gitignored. Each
    clone runs this script once:

        pwsh -File scripts/get-ubc.ps1

    Re-running is free: an already-correct binary is detected and left alone.
    Pass -Force to download again anyway.

.PARAMETER Force
    Download and reinstall even if the pinned version is already present.
#>

[CmdletBinding()]
param(
    [switch] $Force
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# ---------------------------------------------------------------------------
# The pin.
#
# 0.31.2b1 is a PRE-RELEASE, and that is forced rather than chosen: `ubc query
# cypher` is a design commitment for this project and openCypher only arrived in
# 0.31.1b1. Newest stable (0.30.3) offers `query filter` alone. Revisit when
# 0.31.x goes stable.
#
# THE SAME VERSION HAS THREE SPELLINGS. This is the trap in any "is it already
# installed?" check, so both forms we need are recorded literally rather than
# derived from one another:
#
#     git tag                  v0.31.2b1
#     URL path and filename    0.31.2b1        <- $Version
#     `ubc --version` output   ubc 0.31.2-beta.1   <- $VersionString
# ---------------------------------------------------------------------------

$Version       = '0.31.2b1'
$VersionString = 'ubc 0.31.2-beta.1'

# ---------------------------------------------------------------------------
# Checksums.
#
# useblocks publishes no checksum file - .sha256, .sha256sum, SHA256SUMS and
# checksums.txt all return 403 next to the binary (probed 2026-08-16). So these
# hashes were computed from a first download rather than obtained from the
# vendor.
#
# BE CLEAR ABOUT WHAT THAT DOES AND DOES NOT BUY. It is trust-on-first-use: it
# PINS the artefact - a silently replaced or truncated download is caught, and
# every later clone provably gets the same bytes this project was developed
# against. It does NOT authenticate the artefact against useblocks. The real
# risk it addresses is mundane: a pre-release artefact being rebuilt or removed
# under a URL we depend on.
#
# Verified 2026-08-16; sizes cross-checked against the servers' Content-Length.
# ---------------------------------------------------------------------------

$Sha256 = @{
    'windows-x64' = '3DFEFA2F33B29182DB7CF1CCD3C1114A7F8E05BF902D9578A6D2988C9BA27550'  # 69,082,112 bytes
    'linux-x64'   = 'D7121814E8747BEDBACC8F0AA89EB908482CC02D98568563C7E658C5DF193E61'  # 60,413,416 bytes
}

$Platform = 'windows-x64'
$BaseUrl  = 'https://download.useblocks.com/ubc'

# Repository root, derived from this script's own location rather than from git,
# so the script works in an archive export or a copy with no .git directory.
$RepoRoot  = Split-Path -Parent $PSScriptRoot
$ToolsDir  = Join-Path $RepoRoot 'tools'
$Target    = Join-Path $ToolsDir 'ubc.exe'

function Get-InstalledVersion {
    param([string] $Path)

    if (-not (Test-Path -LiteralPath $Path)) { return $null }

    try {
        # A binary that exists but cannot run (truncated, wrong architecture,
        # blocked by policy) must read as "not installed", not crash the script.
        return (& $Path --version 2>$null | Select-Object -First 1).Trim()
    } catch {
        return $null
    }
}

# --- already installed? -----------------------------------------------------

$installed = Get-InstalledVersion -Path $Target

if (-not $Force -and $installed -eq $VersionString) {
    Write-Host "get-ubc: $VersionString already installed at $Target"
    exit 0
}

if ($installed) {
    Write-Host "get-ubc: replacing $installed with $VersionString"
}

# --- download ---------------------------------------------------------------

$fileName = "ubc-$Platform-$Version.exe"
$url      = "$BaseUrl/$Version/$fileName"
$expected = $Sha256[$Platform]

New-Item -ItemType Directory -Force -Path $ToolsDir | Out-Null

# Download beside the target and move into place only after the hash checks out,
# so an interrupted or corrupt download can never leave a half-written binary
# where the pre-commit hook and CI expect a working one.
$temp = "$Target.download"
if (Test-Path -LiteralPath $temp) { Remove-Item -LiteralPath $temp -Force }

Write-Host "get-ubc: downloading $fileName (~66 MB)"

# Invoke-WebRequest renders a progress bar per chunk, which dominates the
# runtime of a 66 MB download in Windows PowerShell - suppressing it is a large
# speedup, not a cosmetic choice.
$previousProgress = $ProgressPreference
$ProgressPreference = 'SilentlyContinue'
try {
    # Windows PowerShell 5.1 can still default to TLS 1.0 depending on the host;
    # download.useblocks.com requires 1.2. Harmless where 1.2 is already set.
    [Net.ServicePointManager]::SecurityProtocol =
        [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12

    Invoke-WebRequest -Uri $url -OutFile $temp -UseBasicParsing
} finally {
    $ProgressPreference = $previousProgress
}

# --- verify -----------------------------------------------------------------

$actual = (Get-FileHash -LiteralPath $temp -Algorithm SHA256).Hash

if ($actual -ne $expected) {
    Remove-Item -LiteralPath $temp -Force
    Write-Error @"
get-ubc: SHA-256 mismatch for $fileName - refusing to install.

    expected  $expected
    actual    $actual

The pinned artefact is not the one this project was built against. Do not work
around this by editing the hash: find out why it changed first. The likely
causes are a rebuilt or replaced pre-release artefact upstream, or a corrupt or
intercepted download.
"@
    exit 1
}

Move-Item -LiteralPath $temp -Destination $Target -Force

# --- confirm ----------------------------------------------------------------

$installed = Get-InstalledVersion -Path $Target

if ($installed -ne $VersionString) {
    Write-Error "get-ubc: installed binary reports '$installed', expected '$VersionString'"
    exit 1
}

Write-Host "get-ubc: installed $VersionString at $Target"
