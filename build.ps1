param([switch]$LocalToolchain)
$ErrorActionPreference = 'Stop'
Set-Location -LiteralPath $PSScriptRoot
if ($LocalToolchain) {
    if (-not (Test-Path -LiteralPath '.tools\rust\bin\cargo.exe')) {
        python tools\bootstrap_rust.py
        if ($LASTEXITCODE -ne 0) { throw 'Rust bootstrap failed' }
    }
    python tools\cargo_local.py test --no-default-features --locked
    if ($LASTEXITCODE -ne 0) { throw 'Tests failed' }
    python tools\cargo_local.py build --release --locked
} else {
    cargo test --no-default-features --locked
    if ($LASTEXITCODE -ne 0) { throw 'Tests failed' }
    cargo build --release --locked
}
if ($LASTEXITCODE -ne 0) { throw 'Release build failed' }
New-Item -ItemType Directory -Path 'bin' -Force | Out-Null
Copy-Item -LiteralPath 'target\release\deskbox-reconstructed.exe' -Destination 'bin\deskbox-reconstructed.exe'
if (Test-Path -LiteralPath 'target\release\WebView2Loader.dll') {
    Copy-Item -LiteralPath 'target\release\WebView2Loader.dll' -Destination 'bin\WebView2Loader.dll'
}
Write-Host "Built: $PSScriptRoot\bin\deskbox-reconstructed.exe"
if (Test-Path -LiteralPath 'target\release\media-tools') {
    Copy-Item -LiteralPath 'target\release\media-tools' -Destination 'bin' -Recurse -Force
}
