.PHONY: test
test:
	turnt -e test -j $(nproc) lessons/*/*.bril playground/*.bril
	turnt -e insn-counter -j $(nproc) lessons/2/*.bril