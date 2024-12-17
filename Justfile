set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

test-cov:
    cargo llvm-cov --lcov --output-path coverage.lcov 2>&1

test-cov-watch:
    watchexec --exts rs --clear --restart 'just test-cov'
