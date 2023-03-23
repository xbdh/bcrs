rust:
	cargo build
run1:rust
	RUST_LOG=info ./target/debug/http --port 3001 --dir "./db/node1"
run2:rust
	RUST_LOG=info ./target/debug/http --port 3002 --dir "./db/node2"

run3:rust
	RUST_LOG=info ./target/debug/http --port 3003 --dir "./db/node3"

clean:
	cargo clean