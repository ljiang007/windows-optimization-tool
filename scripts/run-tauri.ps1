param(
  [Parameter(Mandatory = $true)]
  [ValidateSet('dev', 'build')]
  [string]$Mode
)

$ErrorActionPreference = 'Stop'

$cargoBin = Join-Path $env:USERPROFILE '.cargo\bin'
if (Test-Path $cargoBin) {
  # Rustup installs cargo here; prepend it so new terminals do not depend on refreshed user PATH.
  $env:Path = "$cargoBin;$env:Path"
}

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
  throw 'Cargo was not found. Install Rust with rustup, then reopen PowerShell or keep using this script after installation.'
}

if ($Mode -eq 'dev') {
  npx tauri dev
  exit $LASTEXITCODE
}

npx tauri build
exit $LASTEXITCODE
