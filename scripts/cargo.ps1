# Use an installed Cargo, or the isolated toolchain used for local verification.
# This changes environment variables only for this PowerShell process.
$ErrorActionPreference = 'Stop'
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    $toolRoot = Join-Path $env:LOCALAPPDATA 'ReactCoach11\tools'
    $cargoBin = Join-Path $toolRoot 'cargo\bin'
    if (-not (Test-Path -LiteralPath (Join-Path $cargoBin 'cargo.exe'))) {
        throw 'Install Rust with rustfmt and Clippy, then retry. See README.md.'
    }
    $env:CARGO_HOME = Join-Path $toolRoot 'cargo'
    $env:RUSTUP_HOME = Join-Path $toolRoot 'rustup'
    $env:PATH = $cargoBin + ';' + $env:PATH
    $mingwRoot = Get-ChildItem -LiteralPath $toolRoot -Directory -Filter 'llvm-mingw-*-msvcrt-x86_64' |
        Sort-Object Name -Descending | Select-Object -First 1
    if ($mingwRoot) {
        $env:PATH = (Join-Path $mingwRoot.FullName 'bin') + ';' + $env:PATH
        # Rust ships its GCC runtime libraries; the portable LLVM toolchain
        # supplies dlltool. Keep linking against Rust's matching runtime.
        if (-not $env:CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS) {
            $env:CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS = '-C link-self-contained=yes'
        }
    }
}

& cargo @args
exit $LASTEXITCODE
