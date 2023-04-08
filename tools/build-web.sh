# clean-up
rm -rf web/build
rm -rf web/res
# copy resources
cp -R ./res ./web/res
# build rust app
wasm-pack build --target web --out-dir web/build --release
if [ $? -eq 0 ]; then
  # build succeeded
  if [[ $OSTYPE == 'darwin'* ]]; then
    say "Build succeeded"
  fi
  echo "Build succeeded"
  # start server
  cd web
  http-server -o web
else
  # build failed
  if [[ $OSTYPE == 'darwin'* ]]; then
    say "Build failed"
  fi
  echo "Build failed"
fi