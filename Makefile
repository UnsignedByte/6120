.PHONY: test
test:
	turnt -e test -j $(nproc) lessons/*/*.bril playground/*.bril --save
	turnt -e insn-counter -j $(nproc) lessons/2/*.bril --save
	turnt -e trivial-dce -j $(nproc) lessons/3/*.bril --save