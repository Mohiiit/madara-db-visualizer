$ErrorActionPreference = "Stop"

$Repo = $env:MAKIMONO_REPO
if (-not $Repo) { $Repo = "Mohiiit/makimono" }

$Tag = $env:MAKIMONO_BOOTSTRAPPER_TAG
if (-not $Tag) { $Tag = "makimono" }

$Prefix = $env:MAKIMONO_INSTALL_PREFIX
if (-not $Prefix) {
  $Prefix = Join-Path $HOME "AppData\Local\makimono\bin"
}
New-Item -ItemType Directory -Force -Path $Prefix | Out-Null

$os = "windows"
$arch = "x64"

$asset = "makimono-$Tag-$os-$arch.zip"
$base = "https://github.com/$Repo/releases/download/$Tag"
$url = "$base/$asset"
$sumsUrl = "$base/SHA256SUMS"

$tmp = New-Item -ItemType Directory -Force -Path ([System.IO.Path]::Combine([System.IO.Path]::GetTempPath(), "makimono-install" + [System.Guid]::NewGuid().ToString()))

try {
  $sumsPath = Join-Path $tmp "SHA256SUMS"
  Invoke-WebRequest -UseBasicParsing -Uri $sumsUrl -OutFile $sumsPath
  $expected = (Get-Content $sumsPath | Where-Object { $_ -match [regex]::Escape($asset) } | ForEach-Object { ($_ -split "\s+")[0] } | Select-Object -First 1)
  if (-not $expected) { throw "SHA256SUMS missing entry for $asset" }

  $zipPath = Join-Path $tmp $asset
  Invoke-WebRequest -UseBasicParsing -Uri $url -OutFile $zipPath

  $actual = (Get-FileHash -Algorithm SHA256 $zipPath).Hash.ToLower()
  if ($actual -ne $expected.ToLower()) { throw "checksum mismatch for $asset" }

  Expand-Archive -Path $zipPath -DestinationPath $tmp -Force
  $bin = Join-Path $tmp "makimono.exe"
  if (-not (Test-Path $bin)) { throw "archive did not contain makimono.exe" }

  Copy-Item -Force $bin (Join-Path $Prefix "makimono.exe")
  Write-Output "installed: $(Join-Path $Prefix 'makimono.exe')"
} finally {
  Remove-Item -Recurse -Force $tmp
}
