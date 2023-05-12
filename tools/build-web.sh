# enable rustflags for WebGPU and build rust app
echo "Enabling unstable web sys API's for WebGPU rendering"
export RUSTFLAGS="--cfg=web_sys_unstable_apis"
RUSTFLAGS="--cfg=web_sys_unstable_apis"
wasm-pack build --target web --out-dir web/build --release

# start server is build succeeded
if [ $? -eq 0 ]; then
  # build succeeded
  if [[ $OSTYPE == 'darwin'* ]]; then
    say "Build succeeded"
  fi
  echo "Build succeeded"
  # start server
  http-server -o web
else
  # build failed
  if [[ $OSTYPE == 'darwin'* ]]; then
    say "Build failed"
  fi
  echo "Build failed"
fi