param([string]$Origin = 'http://localhost:5173')
$ErrorActionPreference = 'Stop'
$session = New-Object Microsoft.PowerShell.Commands.WebRequestSession
$headers = @{ Origin = $Origin }
$boot = Invoke-RestMethod -Method Post -Uri "$Origin/api/v1/auth/dev/bootstrap" -Headers $headers -WebSession $session -ContentType 'application/json' -Body '{}'
$headers['X-CSRF-Token'] = $boot.csrf_token
$me = Invoke-RestMethod -Uri "$Origin/api/v1/auth/me" -WebSession $session
if ($me.user.display_name -ne 'Juniper Fox') { throw 'Expected default fictional HR identity' }
if (-not $me.dev_mode) { throw 'Expected development auth' }
foreach ($endpoint in @('/health/live','/health/ready','/api/v1/dashboard','/api/v1/users','/api/v1/organization/tree','/api/v1/terms/applicable','/api/v1/positions','/api/v1/applications','/api/v1/interviews','/api/v1/saved-lists','/api/v1/audit-events')) {
    $null = Invoke-RestMethod -Uri "$Origin$endpoint" -WebSession $session
    Write-Output "PASS $endpoint"
}
Write-Output 'PASS development bootstrap and authenticated feature reads.'
