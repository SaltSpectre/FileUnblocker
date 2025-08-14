param(
    [string]$Version = "",
    [switch]$Clean,
    [switch]$Verbose
)

function Get-ProjectVersion {
    if ($Version) {
        return $Version
    }
    
    # Try to get version from Cargo.toml
    $cargoPath = "../rust/Cargo.toml"
    if (Test-Path $cargoPath) {
        $cargoContent = Get-Content $cargoPath
        $versionLine = $cargoContent | Where-Object { $_ -match 'version\s*=\s*"([^"]+)"' }
        if ($versionLine) {
            return $Matches[1]
        }
    }
    
    # Fallback to date-based version
    $date = Get-Date -Format "yy.MM.dd"
    return "$date.0"
}

function Test-Prerequisites {
    Write-Host "Checking prerequisites..." -ForegroundColor Cyan
    
    # Check .NET SDK
    try {
        $dotnetVersion = dotnet --version
        Write-Host "✓ .NET SDK: $dotnetVersion" -ForegroundColor Green
    } catch {
        Write-Host "✗ .NET SDK not found" -ForegroundColor Red
        return $false
    }
    
    # Check WiX
    try {
        $wixVersion = wix --version
        Write-Host "✓ WiX Toolset: $wixVersion" -ForegroundColor Green
    } catch {
        Write-Host "✗ WiX Toolset not found" -ForegroundColor Red
        Write-Host "  Install with: dotnet tool install --global wix" -ForegroundColor Yellow
        return $false
    }
    
    # Check Rust executables
    $x64Path = "../rust/target/x86_64-pc-windows-msvc/release/unblocker.exe"
    $arm64Path = "../rust/target/aarch64-pc-windows-msvc/release/unblocker.exe"
    
    if (Test-Path $x64Path) {
        Write-Host "✓ AMD64 executable found" -ForegroundColor Green
    } else {
        Write-Host "✗ AMD64 executable not found: $x64Path" -ForegroundColor Red
        Write-Host "  Run: cd ../rust && cargo build --release --target x86_64-pc-windows-msvc" -ForegroundColor Yellow
        return $false
    }
    
    if (Test-Path $arm64Path) {
        Write-Host "✓ ARM64 executable found" -ForegroundColor Green
    } else {
        Write-Host "⚠ ARM64 executable not found: $arm64Path" -ForegroundColor Yellow
        Write-Host "  Run: cd ../rust && cargo build --release --target aarch64-pc-windows-msvc" -ForegroundColor Yellow
        Write-Host "  Continuing without ARM64 support..." -ForegroundColor Yellow
    }
    
    return $true
}

function Build-Installer {
    param([string]$ProjectVersion)
    
    Write-Host "Building installer with version: $ProjectVersion" -ForegroundColor Green
    
    # Clean previous builds
    if ($Clean -and (Test-Path "bin")) {
        Write-Host "Cleaning previous builds..." -ForegroundColor Yellow
        Remove-Item -Recurse -Force "bin"
    }
    
    # Build arguments
    $buildArgs = @(
        "build"
        "-p:VERSION=$ProjectVersion"
    )
    
    if ($Verbose) {
        $buildArgs += "-v", "detailed"
    }
    
    # Build the MSI
    Write-Host "Building MSI package..." -ForegroundColor Cyan
    & dotnet @buildArgs
    
    if ($LASTEXITCODE -eq 0) {
        $msiPath = "bin\Release\en-US\SaltSpectre-Unblocker.msi"
        if (Test-Path $msiPath) {
            $finalName = "SaltSpectre-Unblocker-$ProjectVersion.msi"
            Copy-Item $msiPath $finalName
            Write-Host "✓ Installer built successfully: $finalName" -ForegroundColor Green
            
            # Show file info
            $fileInfo = Get-Item $finalName
            Write-Host "  Size: $([math]::Round($fileInfo.Length / 1MB, 2)) MB" -ForegroundColor Gray
            Write-Host "  Path: $(Resolve-Path $finalName)" -ForegroundColor Gray
        } else {
            Write-Host "✗ MSI file not found at expected location: $msiPath" -ForegroundColor Red
            exit 1
        }
    } else {
        Write-Host "✗ Build failed with exit code $LASTEXITCODE" -ForegroundColor Red
        exit $LASTEXITCODE
    }
}

# Main execution
Write-Host "=== SaltSpectre File Unblocker - MSI Installer Build ===" -ForegroundColor Magenta

if (-not (Test-Prerequisites)) {
    Write-Host "Prerequisites check failed. Cannot continue." -ForegroundColor Red
    exit 1
}

$projectVersion = Get-ProjectVersion
Write-Host "Using version: $projectVersion" -ForegroundColor Cyan

Build-Installer -ProjectVersion $projectVersion

Write-Host "Build completed successfully!" -ForegroundColor Green