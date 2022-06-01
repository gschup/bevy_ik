# Memo

cargo build --release --target wasm32-unknown-unknown
wasm-bindgen .\target\wasm32-unknown-unknown\release\humanoid_ik.wasm --target web --out-dir docs --no-typescript
