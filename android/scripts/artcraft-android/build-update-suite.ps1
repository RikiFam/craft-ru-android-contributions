$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$output = Join-Path $root 'tmp\artcraft\android'
$results = @()
foreach ($app in @('launcher','cadcraft','deckcraft','designcraft','effectcraft','figcraft','filmcraft','gridcraft','lightcraft','pdfcraft','soundcraft','vectorcraft','wordcraft')) {
    $log = Join-Path $output "$app-update-build.log"
    try {
        & "$PSScriptRoot\build-suite.ps1" -App $app *> $log
        $results += [pscustomobject]@{ app=$app; built=$true }
    } catch {
        $_ | Out-String | Add-Content -LiteralPath $log -Encoding UTF8
        $results += [pscustomobject]@{ app=$app; built=$false; error=$_.Exception.Message }
    }
    $results | ConvertTo-Json -Depth 4 | Set-Content (Join-Path $output 'update-build-results.json') -Encoding UTF8
    Write-Output "$app $($results[-1].built)"
}
