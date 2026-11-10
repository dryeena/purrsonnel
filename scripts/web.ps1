param([Parameter(ValueFromRemainingArguments=$true)][string[]]$NpmArguments)
$ErrorActionPreference='Stop'
$projectRoot=Split-Path -Parent $PSScriptRoot
$toolRoot=Join-Path $projectRoot '.tools'
$nodeRoot=Join-Path $toolRoot 'node-v24.22.2-win-arm64'
$node=Join-Path $nodeRoot 'node.exe'
$npm=Join-Path $nodeRoot 'node_modules/npm/bin/npm-cli.js'
if(-not(Test-Path -LiteralPath $node) -or -not(Test-Path -LiteralPath $npm)){
    throw 'Project-local Node is absent. Use Docker, or provision the portable tools with explicit authorization. See docs/FRONTEND.md.'
}
$null=New-Item -ItemType Directory -Force -Path $toolRoot,(Join-Path $toolRoot 'tmp')
foreach($file in @('npmrc','npm-globalrc')){
    $path=Join-Path $toolRoot $file
    if(-not(Test-Path -LiteralPath $path)){[IO.File]::WriteAllText($path,'',[Text.UTF8Encoding]::new($false))}
}
# This script is invoked in a disposable PowerShell process. All tool writes stay
# under the workspace, without user/global npm configuration or persistent PATH edits.
$env:PATH=$nodeRoot+';'+$env:PATH
$env:TEMP=Join-Path $toolRoot 'tmp'
$env:TMP=$env:TEMP
$env:npm_config_cache=Join-Path $toolRoot 'npm-cache'
$env:npm_config_userconfig=Join-Path $toolRoot 'npmrc'
$env:npm_config_globalconfig=Join-Path $toolRoot 'npm-globalrc'
$env:npm_config_prefix=Join-Path $toolRoot 'npm-global'
$env:npm_config_update_notifier='false'
$env:XDG_CACHE_HOME=Join-Path $toolRoot 'cache'
$env:NODE_REPL_HISTORY=Join-Path $toolRoot 'node-repl-history'
Push-Location (Join-Path $projectRoot 'apps/web')
try { & $node $npm @NpmArguments; $result=$LASTEXITCODE }
finally { Pop-Location }
exit $result
