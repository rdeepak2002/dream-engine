# clean-up
rm -rf web/build
rm -rf web/examples
# copy resources
cp -R ./examples ./web/examples

# enable rustflags for WebGPU
echo "Enabling unstable web sys API's for WebGPU rendering"
export RUSTFLAGS="--cfg=web_sys_unstable_apis"
RUSTFLAGS="--cfg=web_sys_unstable_apis"

# build rust app
wasm-pack build --target web --out-dir web/build --release
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