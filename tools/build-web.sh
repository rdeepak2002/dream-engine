rustup run nightly-2024-01-11 wasm-pack build --target web --out-dir ../../web/build --release
rustup run nightly-2024-01-11 wasm-pack build --target web --out-dir ../../web/build-webgl --release --features=wgpu/webgl

# start server is build succeeded
if [ $? -eq 0 ]; then
  # build succeeded
  if [[ $OSTYPE == 'darwin'* ]]; then
    say "Build succeeded"
  fi
  echo "Build succeeded"
  # start server
  cd ..
  cd ..
  cd web
  npm i
  npm run start
else
  # build failed
  if [[ $OSTYPE == 'darwin'* ]]; then
    say "Build failed"
  fi
  echo "Build failed"
fi