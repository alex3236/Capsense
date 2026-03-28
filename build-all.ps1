$ReleaseDir = "target/release"

# Build headless version without default features
Write-Host "`nBuilding Capsense-headless..." -ForegroundColor Green
cargo build --release --no-default-features

if ($LASTEXITCODE -ne 0) {
    Write-Host "Error building headless version!" -ForegroundColor Red
    exit 1
}

# Rename headless version
$HeadlessExe = "$ReleaseDir/Capsense.exe"
$HeadlessTarget = "$ReleaseDir/Capsense-headless.exe"
if (Test-Path $HeadlessExe) {
    Move-Item -Path $HeadlessExe -Destination $HeadlessTarget -Force
    Write-Host "Renamed headless version to Capsense-headless.exe" -ForegroundColor Gray
}

# Build GUI version with default features
Write-Host "Building Capsense (with GUI)..." -ForegroundColor Green
cargo build --release

if ($LASTEXITCODE -ne 0) {
    Write-Host "Error building GUI version!" -ForegroundColor Red
    exit 1
}

Write-Host "`n✓ Both versions built successfully!" -ForegroundColor Green
Write-Host "Output files:" -ForegroundColor Cyan
Write-Host "  - GUI version: target/release/Capsense.exe" -ForegroundColor Yellow
Write-Host "  - Headless version: target/release/Capsense-headless.exe" -ForegroundColor Yellow

