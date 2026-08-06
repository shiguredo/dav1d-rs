.PHONY: test cover check clippy fmt clean

# 全テストを実行する (source-build feature で dav1d をソースからビルドする)
test:
	cargo test --workspace --features source-build

# 全テストカバレッジ付きで実行する
# 計測データの蓄積を防ぐため、実行前にクリアする
cover:
	cargo llvm-cov clean --workspace
	cargo llvm-cov --tests --workspace --features source-build

# cargo check を実行する (prebuilt 経路の検証用)
check:
	cargo check --workspace

# cargo clippy を実行する
clippy:
	cargo clippy --workspace --all-targets --features source-build -- -D warnings

# cargo fmt を実行する
fmt:
	cargo fmt --all

# ビルド成果物を削除する
clean:
	cargo clean
