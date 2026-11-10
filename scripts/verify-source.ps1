param([string]$Root = (Split-Path -Parent $PSScriptRoot))
$ErrorActionPreference = 'Stop'
$checks = 0
function Assert-Source([bool]$Condition, [string]$Message) {
    if (-not $Condition) { throw "Source check failed: $Message" }
    $script:checks++
}
$required = @('README.md','.env.example','docker-compose.yml','apps/api/Cargo.toml','apps/api/src/main.rs','apps/api/src/lib.rs','apps/api/src/app.rs','apps/web/package.json','apps/web/tsconfig.json','apps/web/index.html','docs/BUILD_PROGRESS.md','docs/API.md','docs/AUTH_INTEGRATION.md')
foreach ($file in $required) { Assert-Source (Test-Path -LiteralPath (Join-Path $Root $file)) "Missing $file" }
foreach ($file in @('apps/web/package.json','apps/web/tsconfig.json')) {
    $null = Get-Content -Raw -Encoding UTF8 -LiteralPath (Join-Path $Root $file) | ConvertFrom-Json
    $checks++
}
$originalLogo = (Get-FileHash -LiteralPath (Join-Path $Root 'Purrsonnel Cat HR Logo.png')).Hash
$appLogo = (Get-FileHash -LiteralPath (Join-Path $Root 'apps/web/public/branding/purrsonnel-logo.png')).Hash
Assert-Source ($originalLogo -eq $appLogo) 'Primary logo was not preserved exactly'
$originalIcon = (Get-FileHash -LiteralPath (Join-Path $Root 'purrsonnel-favicon(1).svg')).Hash
$appIcon = (Get-FileHash -LiteralPath (Join-Path $Root 'apps/web/public/branding/favicon.svg')).Hash
Assert-Source ($originalIcon -eq $appIcon) 'Favicon was not preserved exactly'
[xml]$svg = Get-Content -Raw -Encoding UTF8 -LiteralPath (Join-Path $Root 'apps/web/public/branding/favicon.svg')
Assert-Source ($svg.DocumentElement.LocalName -eq 'svg') 'Favicon must be valid SVG'
$manifest = Get-Content -Raw -Encoding UTF8 -LiteralPath (Join-Path $Root 'apps/web/public/manifest.webmanifest') | ConvertFrom-Json
Assert-Source ($manifest.name -eq 'Purrsonnel' -and $manifest.display -eq 'standalone') 'Home-screen name/display'
Assert-Source ($manifest.id -eq '/' -and $manifest.scope -eq '/' -and $manifest.start_url -eq '/') 'Stable application identity and safe start URL'
Add-Type -AssemblyName System.Drawing
foreach ($icon in $manifest.icons) {
    $imagePath = Join-Path $Root ('apps/web/public/' + $icon.src.TrimStart('/'))
    Assert-Source (Test-Path -LiteralPath $imagePath) "Missing app icon $($icon.src)"
    $image = [System.Drawing.Image]::FromFile($imagePath)
    try { Assert-Source ($icon.sizes -eq "$($image.Width)x$($image.Height)") "Icon dimensions do not match manifest: $($icon.src)" }
    finally { $image.Dispose() }
}
Assert-Source (@($manifest.icons | Where-Object { $_.purpose -eq 'maskable' -and $_.sizes -eq '512x512' }).Count -ge 1) 'Missing maskable home-screen icon'
$appleIcon = [System.Drawing.Image]::FromFile((Join-Path $Root 'apps/web/public/branding/apple-touch-icon.png'))
try { Assert-Source ($appleIcon.Width -eq 180 -and $appleIcon.Height -eq 180) 'Apple touch icon dimensions' }
finally { $appleIcon.Dispose() }
$html = Get-Content -Raw -Encoding UTF8 -LiteralPath (Join-Path $Root 'apps/web/index.html')
foreach ($marker in @('/manifest.webmanifest','/branding/apple-touch-icon.png','viewport-fit=cover','apple-mobile-web-app-capable')) { Assert-Source ($html.Contains($marker)) "Missing home-screen metadata: $marker" }
foreach ($asset in @('sw.js','offline.html','offline.css')) { Assert-Source (Test-Path -LiteralPath (Join-Path $Root ('apps/web/public/' + $asset))) "Missing public PWA asset $asset" }
$worker = Get-Content -Raw -Encoding UTF8 -LiteralPath (Join-Path $Root 'apps/web/public/sw.js')
Assert-Source ($worker.Contains("url.pathname.startsWith('/api')")) 'Worker must exclude API routes'
Assert-Source (-not $worker.Contains('cache.put(')) 'Private runtime responses must not be added to the offline cache'
Assert-Source ($worker.Contains("self.addEventListener('push'") -and $worker.Contains('showNotification(')) 'Missing visible push notification handler'
Assert-Source ($worker.Contains('vibrate: [200, 100, 200]')) 'Notification must request a vibration pattern'
$src = Join-Path $Root 'apps/web/src'
foreach ($file in (Get-ChildItem -LiteralPath $src -Recurse -File | Where-Object { $_.Extension -in @('.ts','.tsx') })) {
    $content = Get-Content -Raw -Encoding UTF8 -LiteralPath $file.FullName
    foreach ($match in [regex]::Matches($content, "from\s+['\""'](\.[^'\""']+)['\""']")) {
        $base = Join-Path $file.DirectoryName $match.Groups[1].Value
        $exists = (Test-Path -LiteralPath $base) -or (Test-Path -LiteralPath ($base + '.ts')) -or (Test-Path -LiteralPath ($base + '.tsx')) -or (Test-Path -LiteralPath (Join-Path $base 'index.ts'))
        Assert-Source $exists "Unresolved local import $($match.Groups[1].Value) in $($file.Name)"
    }
    Assert-Source (-not $content.Contains('localStorage.setItem')) "Browser credentials may not be stored in localStorage: $($file.Name)"
    Assert-Source (-not $content.Contains('dangerouslySetInnerHTML')) "Untrusted HTML rendering in $($file.Name)"
}
$sql = Get-Content -Raw -Encoding UTF8 -LiteralPath (Join-Path $Root 'apps/api/migrations/0001_schema.sql')
foreach ($table in @('users','permission_grants','sessions','terms','term_versions','term_signatures','volunteer_entries','volunteer_approval_tokens','volunteer_decision_events','position_postings','applications','interviews','interview_participants','interview_notes','application_final_decisions','saved_lists','bulk_change_jobs','audit_events')) {
    Assert-Source ($sql.Contains("CREATE TABLE $table ")) "Missing database table $table"
}
foreach ($guard in @('immutable_audit','immutable_signatures','immutable_final_decisions','protect_terms','one_active_membership','one_primary_membership','one_active_application')) { Assert-Source ($sql.Contains($guard)) "Missing invariant $guard" }
$routeText = Get-Content -Raw -Encoding UTF8 -LiteralPath (Join-Path $Root 'apps/api/src/app.rs')
$routes = [regex]::Matches($routeText, '\.route\("([^"\r\n]+)"') | ForEach-Object { $_.Groups[1].Value }
$duplicates = $routes | Group-Object | Where-Object { $_.Count -gt 1 }
Assert-Source ($null -eq $duplicates) 'Duplicate Axum route registration'
$tests = Get-ChildItem -LiteralPath (Join-Path $Root 'apps/api/tests') -Recurse -Filter '*.rs'
$dbTests = 0; $unitTests = 0
foreach ($file in $tests) { $text = Get-Content -Raw -LiteralPath $file.FullName; $dbTests += [regex]::Matches($text,'#\[sqlx::test').Count; $unitTests += [regex]::Matches($text,'#\[test\]').Count }
Assert-Source ($dbTests -ge 20) 'Expected at least 20 PostgreSQL workflow tests'
Assert-Source ($unitTests -ge 15) 'Expected at least 15 Rust unit tests'
$apiSource = Join-Path $Root 'apps/api/src'
$release = Get-Content -Raw -Encoding UTF8 -LiteralPath (Join-Path $Root 'docker-compose.release.yml')
Assert-Source ($release.Contains('127.0.0.1:${RELEASE_PORT:-8080}:8080')) 'Release web must be loopback-only'
$privateServices = $release.Substring($release.IndexOf('  db:'),$release.IndexOf('  web:')-$release.IndexOf('  db:'))
Assert-Source (-not $privateServices.Contains('ports:')) 'Release database/API ports must remain private'
Assert-Source ([regex]::Matches($release,'target: release').Count -eq 2) 'Release stack must select both runtime image stages'
Assert-Source ($release.Contains('release_postgres_data:/var/lib/postgresql/data')) 'Missing independent persistent release database'
$webDocker = Get-Content -Raw -Encoding UTF8 -LiteralPath (Join-Path $Root 'apps/web/Dockerfile')
Assert-Source ($webDocker.Contains('FROM nginx:stable-alpine AS release') -and $webDocker.Contains('/app/dist')) 'Release web must serve compiled frontend assets'
$devCompose = Get-Content -Raw -Encoding UTF8 -LiteralPath (Join-Path $Root 'docker-compose.yml')
Assert-Source ($devCompose.Contains('target: development')) 'Development Compose must deliberately select Vite stage'
foreach ($file in (Get-ChildItem -LiteralPath $apiSource -Recurse -File -Filter '*.rs')) {
    $text = Get-Content -Raw -Encoding UTF8 -LiteralPath $file.FullName
    Assert-Source (-not $text.Contains('sqlx::')) "Direct SQLx use remains in application code: $($file.Name)"
    if ($file.FullName -match '\\features\\|\\auth\\') {
        Assert-Source (-not [regex]::IsMatch($text,'"(?:SELECT |INSERT INTO |UPDATE \w+ SET |DELETE FROM |WITH RECURSIVE )')) "SQL leaked into a workflow/auth module: $($file.Name)"
    }
}
$repositoryPath = Join-Path $apiSource 'repositories'
$symbols = @{}
foreach ($file in (Get-ChildItem -LiteralPath $repositoryPath -Filter '*.rs')) {
    $text = Get-Content -Raw -Encoding UTF8 -LiteralPath $file.FullName
    $constants = @{}
    foreach ($constant in [regex]::Matches($text,'pub\(super\) const ([A-Z0-9_]+): &str = "((?:[^"\\]|\\.)*)";')) { $constants[$constant.Groups[1].Value] = $constant.Groups[2].Value }
    foreach ($function in [regex]::Matches($text,'(?ms)^pub (?:async )?fn (\w+)[^\r\n]*\{\r?\n(.*?)^}')) {
        $symbols[$file.BaseName + '::' + $function.Groups[1].Value] = $true
        $body = $function.Groups[2].Value
        foreach ($parameter in [regex]::Matches($body,'let (p\d+): Value')) {
            Assert-Source ([regex]::Matches($body,'\b'+$parameter.Groups[1].Value+'\b').Count -ge 3) "Unused bound value in $($file.Name): $($function.Groups[1].Value)"
        }
        $raw = [regex]::Match($body,'Statement::from_sql_and_values\(DbBackend::Postgres, postgres::([A-Z0-9_]+), vec!\[([^\]]*)\]\)')
        if ($raw.Success) {
            Assert-Source ($constants.ContainsKey($raw.Groups[1].Value)) "Missing PostgreSQL statement: $($file.Name)"
            $parameters = @([regex]::Matches($constants[$raw.Groups[1].Value],'\$(\d+)') | ForEach-Object { [int]$_.Groups[1].Value })
            $expected = if ($parameters.Count) { ($parameters | Measure-Object -Maximum).Maximum } else { 0 }
            $actual = @([regex]::Matches($raw.Groups[2].Value,'\bp\d+\b')).Count
            Assert-Source ($expected -eq $actual) "SeaORM parameter/value mismatch in $($file.Name): $($function.Groups[1].Value)"
            for ($index=1; $index -le $actual; $index++) { Assert-Source ($raw.Groups[2].Value.Split(',')[$index-1].Trim() -eq "p$index") "Reordered PostgreSQL bind in $($file.Name)" }
        }
    }
}
foreach ($file in (Get-ChildItem -LiteralPath $apiSource -Recurse -Filter '*.rs' | Where-Object { $_.Directory.Name -ne 'repositories' })) {
    $text = Get-Content -Raw -Encoding UTF8 -LiteralPath $file.FullName
    foreach ($call in [regex]::Matches($text,'repositories::(\w+::[a-z]\w+)')) { Assert-Source ($symbols.ContainsKey($call.Groups[1].Value)) "Missing repository operation $($call.Groups[1].Value)" }
}
foreach ($file in (Get-ChildItem -LiteralPath $apiSource -Recurse -File -Filter '*.rs')) {
    $text = Get-Content -Raw -Encoding UTF8 -LiteralPath $file.FullName
    foreach ($query in [regex]::Matches($text, 'sqlx::query(?:_scalar)?(?:::<[^>]*>)?\("((?:[^"\\]|\\.)*)"\)(.*?)\.(?:execute|fetch_one|fetch_optional|fetch_all)\(', [System.Text.RegularExpressions.RegexOptions]::Singleline)) {
        $parameters = [regex]::Matches($query.Groups[1].Value, '\$(\d+)') | ForEach-Object { [int]$_.Groups[1].Value }
        $expected = 0
        if ($parameters) { $expected = ($parameters | Measure-Object -Maximum).Maximum }
        $actual = [regex]::Matches($query.Groups[2].Value, '\.bind\(').Count
        Assert-Source ($expected -eq $actual) "SQL parameter/bind mismatch in $($file.Name): expected $expected, found $actual"
    }
    foreach ($module in [regex]::Matches($text, '(?m)^pub mod ([a-z_]+);')) {
        $base = Join-Path $file.DirectoryName $module.Groups[1].Value
        Assert-Source ((Test-Path -LiteralPath ($base + '.rs')) -or (Test-Path -LiteralPath (Join-Path $base 'mod.rs'))) "Missing Rust module $($module.Groups[1].Value)"
    }
}
Write-Output "PASS: $checks source/asset checks. $($routes.Count) routes; $unitTests Rust unit tests; $dbTests PostgreSQL workflow tests authored."
Write-Output 'This is source verification only. It does not substitute for TypeScript compilation, cargo test, SQL migrations, or browser QA.'
