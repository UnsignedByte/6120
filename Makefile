.PHONY: build test brench
build:
	@cargo build

release:
	@cargo build --release

test: build
	-turnt -e test -e opt -e cfg-dot -e call-dot -e domtree-dot -j $(nproc) lessons/*/test/*.bril playground/*.bril --save
	-turnt -e insn-counter -e diagnostics -j $(nproc)  lessons/2/test/*.bril --save
	-turnt -e trivial-dce -e lvn -e lvn-dce -j $(nproc) lessons/3/test/*.bril --save
	-turnt -e reaching-defs -e live-vars -e interval -e const-prop -e available-expr -j $(nproc) lessons/4/test/*.bril lessons/3/test/*.bril --save

brench: build release
	@! brench brench.toml | tee results.csv | grep -q "incorrect"