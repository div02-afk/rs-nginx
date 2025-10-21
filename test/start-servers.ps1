# PowerShell script to start two Node.js servers on different ports

Write-Host "Starting test servers..." -ForegroundColor Green

# Start server on port 3001
Start-Process powershell -ArgumentList "-NoExit", "-Command", "node server.js 3001"

# Wait a bit before starting the second server
Start-Sleep -Seconds 1

# Start server on port 3002
Start-Process powershell -ArgumentList "-NoExit", "-Command", "node server.js 3002"

Write-Host ""
Write-Host "Test servers started!" -ForegroundColor Green
Write-Host "Server 1: http://127.0.0.1:3001" -ForegroundColor Cyan
Write-Host "Server 2: http://127.0.0.1:3002" -ForegroundColor Cyan
Write-Host ""
Write-Host "Endpoints:" -ForegroundColor Yellow
Write-Host "  /health - Health check"
Write-Host "  /hello  - Hello message"
Write-Host ""
Write-Host "Press Ctrl+C in the server windows to stop them"
