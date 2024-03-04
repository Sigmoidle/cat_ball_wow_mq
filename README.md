# cat_ball_wow_mq

Cat ball wow! Inspired by https://goodgis.fun. Written in Rust's Macroquad engine.

Run: `cargo run`
Build: `cargo build --release`

Web Build: `cargo build --release --target wasm32-unknown-unknown`
Web Run (after web build): `basic-http-server -a 0.0.0.0:8080`

Deploy to itch.io (after web build): Copy cat_ball_wow_mq.wasm from target/ to dist. Then add the assets folder to dist. Then zip dist.
