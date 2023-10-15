start "" "%DF_EXECUTABLE%"
rem timeout 5 > nul
cls
cargo +nightly build --target=i686-pc-windows-msvc --lib %*
cargo +nightly run --target=i686-pc-windows-msvc --bin dfdecomp %*