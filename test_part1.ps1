# Script de test pour la PARTIE 1
# Usage: .\test_part1.ps1

Write-Host "🧪 Testing Distributed Media Queue - PARTIE 1" -ForegroundColor Cyan
Write-Host ""

# Test 1: Build
Write-Host "📦 Building shared library..." -ForegroundColor Yellow
cargo build -p shared
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Build failed!" -ForegroundColor Red
    exit 1
}
Write-Host "✅ Build successful!" -ForegroundColor Green
Write-Host ""

# Test 2: Unit tests
Write-Host "🧪 Running unit tests..." -ForegroundColor Yellow
cargo test -p shared
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Tests failed!" -ForegroundColor Red
    exit 1
}
Write-Host "✅ All unit tests passed!" -ForegroundColor Green
Write-Host ""

# Test 3: Example
Write-Host "🎯 Running basic usage example..." -ForegroundColor Yellow
cargo run --example basic_usage -p shared
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Example failed!" -ForegroundColor Red
    exit 1
}
Write-Host ""

# Summary
Write-Host "🎉 All tests completed successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Cyan
Write-Host "  1. Start Redis: docker-compose up -d redis" -ForegroundColor White
Write-Host "  2. Run integration tests: cargo test -p shared -- --ignored" -ForegroundColor White
Write-Host "  3. Move to PART 2: API Server implementation" -ForegroundColor White
