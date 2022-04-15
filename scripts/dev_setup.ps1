<#
.SYNOPSIS
Usage:
Installs or updates necessary dev tools for starcoinorg/starcoin.
-p update environment variables only
-t install build tools
-y installs or updates Move prover tools: z3, cvc5, dotnet, boogie
-dir <path> - directory to install to
.DESCRIPTION
helper for setting up starcoin development environment
#>
param(
[Parameter()]
[Alias('p')]
[switch]$INSTALL_PROFILE,

[Parameter()]
[Alias('t')]
[switch]$INSTALL_BUILD_TOOLS,

[Parameter()]
[Alias('y')]
[switch]$INSTALL_PROVER,

[Parameter()]
[Alias('dir')]
[string]$INSTALL_DIR="${HOME}\.starcoin_deps"
)



$Z3_VERSION="4.8.13"
$CVC5_VERSION="1.0.0"
$BOOGIE_VERSION="2.9.6"

Write-Host "INSTALL_PROFILE=$INSTALL_PROFILE"
Write-Host "INSTALL_BUILD_TOOLS=$INSTALL_BUILD_TOOLS"
Write-Host "INSTALL_PROVER=$INSTALL_PROVER"
Write-Host "INSTALL_DIR=$INSTALL_DIR"

# check environment if exist
# $key env $value value
function check_set_env {
    param(
    [Parameter(Mandatory=$true)]
    [string]$key,
    [Parameter(Mandatory=$true)]
    [string]$value
    )
    [string] $env_value = [Environment]::GetEnvironmentVariable($key, 'User')
    if($env_value -ne $value){
        Write-Host "set $key=$value"
        [Environment]::SetEnvironmentVariable($key,$value,'User')
    }else{
        Write-Host "Environment variable $key is set"
    }
}

# set env and path variables
function set_env_path {
    Write-Host "Setting environment variables for profile"
    check_set_env "Z3_EXE" "$INSTALL_DIR\z3\z3.exe"
    check_set_env "CVC5_EXE" "$INSTALL_DIR\cvc5.exe"
    check_set_env "BOOGIE_EXE" "$INSTALL_DIR\tools\boogie\"
}

function install_z3 {
    $z3pkg = "z3-$Z3_VERSION-x64-win"
    # download z3
    Invoke-WebRequest -Uri "https://github.com/Z3Prover/z3/releases/download/z3-$z3_version/$z3pkg.zip" -OutFile "$INSTALL_DIR\z3\z3.zip"
    # unzip z3
    Write-Host "Unzipping z3"
    Expand-Archive "$INSTALL_DIR\z3\z3.zip" -DestinationPath "$INSTALL_DIR\z3"
    # remove z3.zip
    Remove-Item "$INSTALL_DIR\z3\z3.zip" -Force
    # mv z3.exe into z3 path
    Copy-Item "$INSTALL_DIR\z3\$z3pkg\bin\z3.exe" "$INSTALL_DIR\z3\z3.exe"
}

function install_cvc5 {
    Invoke-WebRequest -Uri "https://github.com/cvc5/cvc5/releases/download/cvc5-$CVC5_VERSION/cvc5-Win64.exe" -OutFile "$INSTALL_DIR\cvc5.exe"
}

function install_boogie {
    dotnet tool update --tool-path "$INSTALL_DIR\tools\boogie" Boogie --version $BOOGIE_VERSION
}

function install_build_tools {
    try {
        clang 2>&1>$null
    } catch {
        Write-Error "Clang not found, installing llvm and clang"
        $llvmVersion = "12.0.0"
        Write-Host "Installing LLVM $llvmVersion ..." -ForegroundColor Cyan
        Write-Host "Downloading..."
        $exePath = "$env:temp\LLVM-$llvmVersion-win64.exe"
        Invoke-WebRequest -Uri "https://github.com/llvm/llvm-project/releases/download/llvmorg-$llvmVersion/LLVM-$llvmVersion-win64.exe" -OutFile $exePath
        Write-Host "Installing..."
        cmd /c start $exePath
        Write-Host "Installed" -ForegroundColor Green
        [Environment]::SetEnvironmentVariable("Path", $env:Path + ";C:\Program Files\LLVM\bin", 'User')
    }
    try {
        dotnet 2>&1>$null
    } catch {
        Write-Error "Dotnet sdk not found, installing dotnet sdk!"
        $exePath = "$env:temp\dotnet-sdk-6.0.202-win-x64.exe"
        Invoke-WebRequest -Uri "https://download.visualstudio.microsoft.com/download/pr/e4f4bbac-5660-45a9-8316-0ffc10765179/8ade57de09ce7f12d6411ed664f74eca/dotnet-sdk-6.0.202-win-x64.exe" -OutFile $exePath
        Write-Host "Installing..."
        cmd /c start $exePath
        Write-Host "Installed" -ForegroundColor Green
    }
    try {
        cargo 2>&1>$null
    } catch {
        throw "install rust by yourself please"
    }
}

if ($INSTALL_PROFILE -eq $true) {
    Write-Host "Installing profile"
    set_env_path
}
if ($INSTALL_BUILD_TOOLS -eq $true) {
    Write-Host "Installing build tools"
    install_build_tools
}
if ($INSTALL_PROVER -eq $true) {
    Write-Host "Installing prover"
    install_z3
    install_cvc5
    install_boogie
}
