sudo apt update;
sudo apt install -y rustc cargo;
export HOME=/root;
git config --global credential.helper store;
git clone https://github.com/nkalupahana/kalupana-eece4371-android.git repo;
cd repo;
git checkout final;
cd final/server;
cargo run --release;