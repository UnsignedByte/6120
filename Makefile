.PHONY: build release graphs test brench bench
build:
	@cargo build

release:
	@cargo build --release

test: build
	-turnt -j $(nproc) -c turnt-global.toml --save playground/*.bril lessons/*/test/*.bril
	-turnt -j $(nproc) --save lessons/*/test/*.bril

graphs: build
	-turnt -j $(nproc) -c turnt-global.toml -e cfg-dot -e call-dot -e domtree-dot -e domsets-dot --save playground/*.bril lessons/*/test/*.bril

brench: build
	@! brench brench.toml | tee results.csv | grep -q "incorrect"

bench: brench