# FileBay API Connection Test
$ErrorActionPreference = "Stop"

$token = "7aca7eac68e5936e6366037c401832d3939876fc"
$url = "https://uat-filebay.cheersai.cloud"
$owner = "junqianxi"
$repo = "cheersAI"

Write-Host "=== FileBay API Connection Test ===" -ForegroundColor Cyan
Write-Host "URL: $url" -ForegroundColor Yellow
Write-Host "Owner: $owner" -ForegroundColor Yellow
Write-Host "Repo: $repo" -ForegroundColor Yellow
Write-Host ""

# Test 1: Check if repo exists
$repoUrl = "$url/api/v1/repos/$owner/$repo"
Write-Host "--- Test 1: Check Repository ---" -ForegroundColor Green
Write-Host "Endpoint: $repoUrl"

try {
    $headers = @{
        "Authorization" = "token $token"
        "Accept" = "application/json"
    }
    
    # Try with certificate validation disabled
    if ($PSVersionTable.PSVersion.Major -ge 6) {
        $response = Invoke-RestMethod -Uri $repoUrl -Headers $headers -Method Get -SkipCertificateCheck
    } else {
        # For PowerShell 5.1, disable certificate validation
        [System.Net.ServicePointManager]::ServerCertificateValidationCallback = {$true}
        [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.SecurityProtocolType]::Tls12
        $response = Invoke-RestMethod -Uri $repoUrl -Headers $headers -Method Get
    }
    
    Write-Host "✅ Repository exists!" -ForegroundColor Green
    Write-Host "Repo Name: $($response.name)" -ForegroundColor Cyan
    Write-Host "Full Name: $($response.full_name)" -ForegroundColor Cyan
    Write-Host "Private: $($response.private)" -ForegroundColor Cyan
    Write-Host "Clone URL: $($response.clone_url)" -ForegroundColor Cyan
    
} catch {
    Write-Host "❌ Failed to check repository" -ForegroundColor Red
    Write-Host "Status Code: $($_.Exception.Response.StatusCode.value__)" -ForegroundColor Red
    Write-Host "Error: $($_.Exception.Message)" -ForegroundColor Red
    
    if ($_.Exception.Response) {
        $reader = New-Object System.IO.StreamReader($_.Exception.Response.GetResponseStream())
        $responseBody = $reader.ReadToEnd()
        Write-Host "Response: $responseBody" -ForegroundColor Red
    }
}

Write-Host ""

# Test 2: Get user info
$userUrl = "$url/api/v1/user"
Write-Host "--- Test 2: Get User Info ---" -ForegroundColor Green
Write-Host "Endpoint: $userUrl"

try {
    $headers = @{
        "Authorization" = "token $token"
        "Accept" = "application/json"
    }
    
    if ($PSVersionTable.PSVersion.Major -ge 6) {
        $response = Invoke-RestMethod -Uri $userUrl -Headers $headers -Method Get -SkipCertificateCheck
    } else {
        [System.Net.ServicePointManager]::ServerCertificateValidationCallback = {$true}
        [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.SecurityProtocolType]::Tls12
        $response = Invoke-RestMethod -Uri $userUrl -Headers $headers -Method Get
    }
    
    Write-Host "✅ User authenticated!" -ForegroundColor Green
    Write-Host "Username: $($response.login)" -ForegroundColor Cyan
    Write-Host "Email: $($response.email)" -ForegroundColor Cyan
    Write-Host "Full Name: $($response.full_name)" -ForegroundColor Cyan
    
} catch {
    Write-Host "❌ Failed to get user info" -ForegroundColor Red
    Write-Host "Status Code: $($_.Exception.Response.StatusCode.value__)" -ForegroundColor Red
    Write-Host "Error: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host ""
Write-Host "=== Test Complete ===" -ForegroundColor Cyan

# Reset certificate validation
[System.Net.ServicePointManager]::ServerCertificateValidationCallback = $null
