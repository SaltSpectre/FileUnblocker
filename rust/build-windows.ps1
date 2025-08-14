param(
    [switch]$Clean,
    [switch]$Check,
    [switch]$VerifyToolchain
)

function Test-Toolchain {
    Write-Host "Verifying toolchain..." -ForegroundColor Cyan
    
    # Check Rust installation
    try {
        $rustVersion = rustc --version
        Write-Host "✓ Rust: $rustVersion" -ForegroundColor Green
    } catch {
        Write-Host "✗ Rust not found" -ForegroundColor Red
        return $false
    }
    
    # Check Cargo
    try {
        $cargoVersion = cargo --version
        Write-Host "✓ Cargo: $cargoVersion" -ForegroundColor Green
    } catch {
        Write-Host "✗ Cargo not found" -ForegroundColor Red
        return $false
    }
    
    # Check Visual Studio Build Tools
    $vsInstallPaths = @(
        "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools",
        "C:\Program Files\Microsoft Visual Studio\2022\Community",
        "C:\Program Files\Microsoft Visual Studio\2022\Professional",
        "C:\Program Files\Microsoft Visual Studio\2022\Enterprise"
    )
    
    $vsFound = $false
    foreach ($path in $vsInstallPaths) {
        if (Test-Path "$path\VC\Tools\MSVC") {
            Write-Host "✓ Visual Studio Build Tools found: $path" -ForegroundColor Green
            $vsFound = $true
            break
        }
    }
    
    if (-not $vsFound) {
        Write-Host "✗ Visual Studio Build Tools not found" -ForegroundColor Red
        Write-Host "  Install from: https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022" -ForegroundColor Yellow
    }
    
    # Check Windows SDK
    $sdkPaths = @(
        "C:\Program Files (x86)\Windows Kits\10",
        "C:\Program Files\Windows Kits\10"
    )
    
    $sdkFound = $false
    foreach ($path in $sdkPaths) {
        if (Test-Path "$path\Lib") {
            Write-Host "✓ Windows SDK found: $path" -ForegroundColor Green
            $sdkFound = $true
            break
        }
    }
    
    if (-not $sdkFound) {
        Write-Host "✗ Windows SDK not found" -ForegroundColor Red
    }
    
    # Check installed Rust targets
    $targets = rustup target list --installed | Out-String
    if ($targets -match "x86_64-pc-windows-msvc") {
        Write-Host "✓ AMD64 target installed" -ForegroundColor Green
    } else {
        Write-Host "✗ AMD64 target not installed" -ForegroundColor Yellow
    }
    
    if ($targets -match "aarch64-pc-windows-msvc") {
        Write-Host "✓ ARM64 target installed" -ForegroundColor Green
    } else {
        Write-Host "✗ ARM64 target not installed" -ForegroundColor Yellow
    }
    
    return ($vsFound -and $sdkFound)
}

if ($VerifyToolchain) {
    Test-Toolchain
    exit
}

# Ensure Windows targets are installed
rustup target add x86_64-pc-windows-msvc
rustup target add aarch64-pc-windows-msvc

if ($Clean -and (Test-Path "target")) {
    Remove-Item -Recurse -Force "target"
}

if ($Check) {
    cargo check --target x86_64-pc-windows-msvc 2>&1 | Tee-Object -FilePath "build-transcript.txt"
} else {
    cargo build --release --target x86_64-pc-windows-msvc 2>&1 | Tee-Object -FilePath "build-transcript.txt"
    cargo build --release --target aarch64-pc-windows-msvc 2>&1 | Tee-Object -FilePath "build-transcript.txt" -Append
}