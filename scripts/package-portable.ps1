param(
    [string]$Version = "0.1.0",
    [string]$ModelCache = ".local\bakeoff\m3",
    [string]$OutputRoot = "dist"
)

$ErrorActionPreference = "Stop"
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$modelRoot = (Resolve-Path (Join-Path $repoRoot $ModelCache)).Path
$outputBase = Join-Path $repoRoot $OutputRoot
$packageName = "Nous-Wave-$Version-windows-x86_64"
$packageRoot = Join-Path $outputBase $packageName
$zipPath = Join-Path $outputBase "$packageName.zip"

if (-not (Test-Path (Join-Path $modelRoot "models--BAAI--bge-m3"))) {
    throw "Selected BAAI/bge-m3 FastEmbed assets are missing from $modelRoot"
}

New-Item -ItemType Directory -Force -Path $outputBase | Out-Null
if (Test-Path $packageRoot) { Remove-Item -LiteralPath $packageRoot -Recurse -Force }
if (Test-Path $zipPath) { Remove-Item -LiteralPath $zipPath -Force }
New-Item -ItemType Directory -Force -Path $packageRoot, (Join-Path $packageRoot "models"), (Join-Path $packageRoot "licenses") | Out-Null

$env:PROTOC = (Resolve-Path (Join-Path $repoRoot ".local\protoc\bin\protoc.exe")).Path
$env:POSTGRESQL_VERSION = "18.6.0"
cargo build --release --features portable-release -p nous-wave
Copy-Item -LiteralPath (Join-Path $repoRoot "target\release\nous-wave.exe") -Destination (Join-Path $packageRoot "nous-wave.exe")
Copy-Item -LiteralPath (Join-Path $repoRoot "config.example.toml") -Destination (Join-Path $packageRoot "config.toml")
Copy-Item -LiteralPath (Join-Path $modelRoot "models--BAAI--bge-m3") -Destination (Join-Path $packageRoot "models") -Recurse

$commit = (git -C $repoRoot rev-parse HEAD).Trim()
$versionData = [ordered]@{
    product = "Nous Wave"
    version = $Version
    target = "windows-x86_64"
    commit = $commit
    postgres = "PostgreSQL 18.6 / postgresql_embedded 0.21.0"
    embedding = [ordered]@{
        model = "BAAI/bge-m3"
        fastembed = "6.0.2"
        revision = "5617a9f61b028005a4858fdac845db406aefb181"
        preprocessing = "identity-l2-v1"
        dimension = 1024
    }
}
$versionData | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $packageRoot "VERSION.json") -Encoding utf8

@"
Nous Wave $Version — Windows x86_64 portable runtime

Run `nous-wave.exe status` or `nous-wave.exe serve` from this directory.
The bundled profile owns PostgreSQL data under data\ and uses the bundled
FastEmbed BAAI/bge-m3 assets under models\. No Rust, Cargo, repository checkout,
system PostgreSQL or embedding service is required.

The package is relocatable as a directory before and after first start. All
operator data stays beneath the package root unless config.toml is changed.
The runtime binds to loopback only.
"@ | Set-Content -LiteralPath (Join-Path $packageRoot "README.txt") -Encoding utf8

@"
Nous Wave runtime notices

PostgreSQL 18.6 is distributed through postgresql_embedded 0.21.0.
FastEmbed 6.0.2 loads BAAI/bge-m3 (Apache-2.0 model license).
Review upstream license texts before redistribution and add the exact notices
required by the selected assets to this directory.
"@ | Set-Content -LiteralPath (Join-Path $packageRoot "licenses\THIRD-PARTY-NOTICES.txt") -Encoding utf8
"PostgreSQL license is distributed with the bundled PostgreSQL 18.6 asset; see the upstream PostgreSQL license notice." | Set-Content -LiteralPath (Join-Path $packageRoot "licenses\PostgreSQL-LICENSE") -Encoding utf8
"BAAI/bge-m3 model license: Apache-2.0. Preserve the license and model card with redistributed assets." | Set-Content -LiteralPath (Join-Path $packageRoot "licenses\bge-m3-LICENSE.txt") -Encoding utf8

Compress-Archive -Path (Join-Path $packageRoot "*") -DestinationPath $zipPath -CompressionLevel Optimal
Write-Output $zipPath
