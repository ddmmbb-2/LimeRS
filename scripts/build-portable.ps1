$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
Set-Location $root
& cargo test -p limers_core
if ($LASTEXITCODE -ne 0) { throw 'Rust core tests failed.' }
& cargo build --release -p limers-portable
if ($LASTEXITCODE -ne 0) { throw 'Windows EXE compilation failed.' }
New-Item -ItemType Directory -Force (Join-Path $root 'portable\data') | Out-Null
Copy-Item (Join-Path $root 'target\release\LimeRS.exe') (Join-Path $root 'portable\LimeRS.exe') -Force
Write-Host 'SUCCESS: portable\LimeRS.exe'
