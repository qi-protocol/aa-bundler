build:
	cargo build --release

run-silius:
	cargo run --release -- bundler --eth-client-address http://127.0.0.1:8545 --mnemonic-file ${HOME}/.silius/0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 --beneficiary 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 --entry-points 0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789 --http --ws

run-silius-bundling:
	cargo run --release -- bundling --eth-client-address http://127.0.0.1:8545 --mnemonic-file ${HOME}/.silius/0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 --beneficiary 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 --entry-points 0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789

run-silius-uopool:
	cargo run --release -- uopool --eth-client-address http://127.0.0.1:8545 --entry-points 0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789

run-silius-rpc:
	cargo run --release -- rpc --http --ws

run-silius-create-wallet:
	cargo run --release -- create-wallet --output-path ${HOME}/.silius

run-silius-debug:
	cargo run --release -- bundler --eth-client-address ws://127.0.0.1:8546 --mnemonic-file ${HOME}/.silius/0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 --beneficiary 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 --entry-points 0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789 --http --ws --http.api eth,debug,web3 --ws.api eth,debug,web3

run-silius-debug-mode:
	cargo run --profile debug-fast -- bundler --verbosity 4 --eth-client-address ws://127.0.0.1:8546 --mnemonic-file /home/vid/.silius/0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 --beneficiary 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 --entry-points 0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789 --http --ws --http.api eth,debug,web3 --ws.api eth,debug,web3

fetch-thirdparty:
	git submodule update --init

setup-thirdparty:
	cd crates/contracts/thirdparty/account-abstraction && yarn install --frozen-lockfile --immutable && yarn compile && cd ../../../..
	cd tests/thirdparty/bundler && yarn install --frozen-lockfile --immutable && yarn preprocess && cd ../../..

test:
	cargo test --workspace

format:
	cargo fmt --all
	cargo sort --workspace

lint:
	cargo fmt --all -- --check
	cargo clippy --all -- -D warnings -A clippy::derive_partial_eq_without_eq -D clippy::unwrap_used -D clippy::uninlined_format_args
	cargo sort --check --workspace
	cargo udeps --workspace

clean:
	cd crates/contracts/thirdparty/account-abstraction && yarn clean && cd ../..
	cd crates/contracts/thirdparty/bundler && yarn clear && cd ../..
	cargo clean
