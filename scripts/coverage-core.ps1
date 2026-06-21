$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$crateRoot = Resolve-Path (Join-Path $repoRoot "crates\alert-timer-core")
$coverageDir = Join-Path $crateRoot "target\coverage"
$coveragePath = [System.IO.Path]::GetFullPath($coverageDir)
$cratePath = [System.IO.Path]::GetFullPath($crateRoot)

if (-not $coveragePath.StartsWith($cratePath, [System.StringComparison]::OrdinalIgnoreCase)) {
  throw "Refusing to clean coverage directory outside crate root: $coveragePath"
}

Remove-Item -LiteralPath $coveragePath -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $coveragePath | Out-Null

$sysroot = (& rustc --print sysroot).Trim()
$hostLine = (& rustc -vV | Select-String "^host:").Line
$hostTriple = $hostLine -replace "^host:\s*", ""
$llvmTools = Join-Path $sysroot "lib\rustlib\$hostTriple\bin"
$llvmProfdata = Join-Path $llvmTools "llvm-profdata.exe"
$llvmCov = Join-Path $llvmTools "llvm-cov.exe"

if (-not (Test-Path $llvmProfdata) -or -not (Test-Path $llvmCov)) {
  throw "llvm-tools-preview is required. Run: rustup component add llvm-tools-preview"
}

$previousRustFlags = $env:RUSTFLAGS
$previousCargoIncremental = $env:CARGO_INCREMENTAL
$previousProfileFile = $env:LLVM_PROFILE_FILE

try {
  $env:RUSTFLAGS = "-C instrument-coverage"
  $env:CARGO_INCREMENTAL = "0"
  $env:LLVM_PROFILE_FILE = Join-Path $coveragePath "%p-%m.profraw"

  Push-Location $crateRoot
  cargo test
  Pop-Location
} finally {
  $env:RUSTFLAGS = $previousRustFlags
  $env:CARGO_INCREMENTAL = $previousCargoIncremental
  $env:LLVM_PROFILE_FILE = $previousProfileFile
}

$rawProfiles = Get-ChildItem -LiteralPath $coveragePath -Filter "*.profraw"
if ($rawProfiles.Count -eq 0) {
  throw "No coverage profiles were generated."
}

$profileData = Join-Path $coveragePath "coverage.profdata"
& $llvmProfdata merge -sparse -o $profileData $rawProfiles.FullName
if ($LASTEXITCODE -ne 0) {
  throw "llvm-profdata failed with exit code $LASTEXITCODE"
}

$testBinaries = Get-ChildItem -LiteralPath (Join-Path $crateRoot "target\debug\deps") -Filter "*.exe" |
  Where-Object { $_.Name -match "^(alert_timer_core|timer_engine_boundaries)-[0-9a-f]+\.exe$" } |
  Group-Object { $_.Name -replace "-[0-9a-f]+\.exe$", "" } |
  ForEach-Object { $_.Group | Sort-Object LastWriteTime -Descending | Select-Object -First 1 }

if (-not $testBinaries -or $testBinaries.Count -eq 0) {
  throw "Could not find core test binaries."
}

$coverageArgs = @("--instr-profile=$profileData", "--ignore-filename-regex=\\tests\\|\\\.cargo\\|\\rustc\\", $testBinaries[0].FullName)
if ($testBinaries.Count -gt 1) {
  foreach ($testBinary in $testBinaries[1..($testBinaries.Count - 1)]) {
    $coverageArgs += "--object=$($testBinary.FullName)"
  }
}

$report = & $llvmCov report @coverageArgs
if ($LASTEXITCODE -ne 0) {
  throw "llvm-cov failed with exit code $LASTEXITCODE"
}

$report | ForEach-Object { Write-Output $_ }

$totalLine = $report | Where-Object { $_ -match "^TOTAL\s+" } | Select-Object -Last 1
if (-not $totalLine) {
  throw "Could not parse coverage TOTAL line."
}

$percentages = [regex]::Matches($totalLine, "\d+(?:\.\d+)?%")
if ($percentages.Count -lt 3) {
  throw "Could not parse line coverage percentage from: $totalLine"
}

$lineCoverage = [double]($percentages[2].Value.TrimEnd("%"))
$threshold = 80.0
Write-Output ("Line coverage: {0:N2}% (threshold {1:N2}%)" -f $lineCoverage, $threshold)

if ($lineCoverage -lt $threshold) {
  throw ("Line coverage {0:N2}% is below threshold {1:N2}%" -f $lineCoverage, $threshold)
}
