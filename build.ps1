$CargoBin = "$env:USERPROFILE\.cargo\bin"
if ($env:Path -notlike "*$CargoBin*") {
    $env:Path = "$CargoBin;" + $env:Path
}

Write-Host "1. Verification de la cible WASM..." -ForegroundColor Cyan
rustup target add wasm32-unknown-unknown

Write-Host "`n2. Compilation de l'extension en mode Release..." -ForegroundColor Cyan
cargo build --release

if ($LASTEXITCODE -ne 0) {
    Write-Host "Erreur lors de la compilation." -ForegroundColor Red
    exit $LASTEXITCODE
}

Write-Host "`n3. Packaging de l'extension en .aix..." -ForegroundColor Cyan
$WasmPath = ".\target\wasm32-unknown-unknown\release\comics_tracker_aidoku.wasm"
$AixDir = ".\build_aix"
$PayloadDir = "$AixDir\Payload"

if (Test-Path $AixDir) { Remove-Item -Recurse -Force $AixDir }
New-Item -ItemType Directory -Path $PayloadDir | Out-Null

Copy-Item $WasmPath -Destination "$PayloadDir\main.wasm"
Copy-Item ".\res\source.json" -Destination "$PayloadDir\source.json"
Copy-Item ".\res\settings.json" -Destination "$PayloadDir\settings.json"

if (Test-Path ".\res\icon.png") {
    Copy-Item ".\res\icon.png" -Destination "$PayloadDir\icon.png"
}

$ZipPath = "$PWD\ComicsTracker.zip"
$AixPath = "$PWD\ComicsTracker.aix"
if (Test-Path $ZipPath) { Remove-Item -Force $ZipPath }
if (Test-Path $AixPath) { Remove-Item -Force $AixPath }

Push-Location $AixDir
tar.exe -a -c -f $ZipPath Payload
Pop-Location

Rename-Item -Path $ZipPath -NewName $AixPath

Remove-Item -Recurse -Force $AixDir

Write-Host "`nSucces ! Le fichier ComicsTracker.aix a ete genere." -ForegroundColor Green
