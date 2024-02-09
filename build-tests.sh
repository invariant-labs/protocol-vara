# Install gear node
curl https://get.gear.rs/gear-v1.0.2-x86_64-unknown-linux-gnu.tar.xz | tar xJ

./gear --dev

cargo test --release
