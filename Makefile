rust:
	cargo build
run1:rust
	RUST_LOG=info  ./target/debug/http --miner "0x446E89D661D607868FBD8E881E6A15C3797AF140" --ip "127.0.0.1" --port 3001 --dir "./db/node1"
run2:rust
	RUST_LOG=info ./target/debug/http  --miner "0xE4DFEB6011C12A097190BBFBFF859979E2616AD0" --ip "127.0.0.1" --port 3002 --dir "./db/node2"

run3:rust
	RUST_LOG=info ./target/debug/http  --miner "jake" --ip "127.0.0.1" --port 3003 --dir "./db/node3"

clean:
	cargo clean