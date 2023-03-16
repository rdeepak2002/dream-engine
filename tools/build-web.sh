rm -rf web/build
wasm-pack build --target web --out-dir web/build --release
if [[ $OSTYPE == 'darwin'* ]]; then
  say "Build succeeded"
fi