
#!/bin/bash
set -e
cd "`dirname $0`"
dir='./res'
if [[ ! -e $dir ]]; then
    mkdir $dir
fi
RUSTFLAGS='-C link-arg=-s' cargo build --all --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/*.wasm $dir