Write-Host "=== FileBay Connection Test ===" -ForegroundColor Cyan
Write-Host ""

$url = "https://uat-filebay.cheersai.cloud"
$apiUrl = "$url/api/v1/repos/junqianxi/cheersAI"

Write-Host "Target URL: $apiUrl" -ForegroundColor Yellow
Write-Host ""

# Test 1: Base URL
Write-Host "--- Test 1: Base URL ($url) ---" -ForegroundColor Green
try {
    $response = Invoke-WebRequest -Uri $url -TimeoutSec 10 -SkipCertificateCheck -ErrorAction Stop
    Write-Host "✅ Success! Status: $($response.StatusCode)" -ForegroundColor Green
    Write-Host "Headers: $($response.Headers | Out-String)"
} catch {
    Write-Host "❌ Failed: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "Error Details: $($_.Exception.Response.StatusCode)" -ForegroundColor Red
}

Write-Host ""

# Test 2: API endpoint
Write-Host "--- Test 2: API Endpoint ---" -ForegroundColor Green
try {
    $response = Invoke-WebRequest -Uri $apiUrl -TimeoutSec 10 -SkipCertificateCheck -ErrorAction Stop
    Write-Host "✅ Success! Status: $($response.StatusCode)" -ForegroundColor Green
    Write-Host "Content preview: $($response.Content.Substring(0, [Math]::Min(200, $response.Content.Length)))"
} catch {
    Write-Host "❌ Failed: $($_.Exception.Message)" -ForegroundColor Red
    if ($_.Exception.Response) {
        Write-Host "Status Code: $($_.Exception.Response.StatusCode.value__)" -ForegroundColor Red
    }
}

Write-Host ""

# Test 3: DNS Resolution
Write-Host "--- Test 3: DNS Resolution ---" -ForegroundColor Green
try {
    $dns = Resolve-DnsName -Name "uat-filebay.cheersai.cloud" -ErrorAction Stop
    Write-Host "✅ DNS Resolved:" -ForegroundColor Green
    $dns | ForEach-Object { Write-Host "  $($_.Name) -> $($_.IPAddress)" }
} catch {
    Write-Host "❌ DNS Failed: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host ""

# Test 4: TCP Connection
Write-Host "--- Test 4: TCP Connection Test (Port 443) ---" -ForegroundColor Green
try {
    $tcpClient = New-Object System.Net.Sockets.TcpClient
    $tcpClient.Connect("uat-filebay.cheersai.cloud", 443)
    if ($tcpClient.Connected) {
        Write-Host "✅ TCP Connection Successful" -ForegroundColor Green
        $tcpClient.Close()
    }
} catch {
    Write-Host "❌ TCP Connection Failed: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host ""
Write-Host "=== Test Complete ===" -ForegroundColor Cyan
