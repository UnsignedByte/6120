.PHONY: test
test:
	turnt -e test -e diagnostics -j $(nproc) lessons/*/test/*.bril playground/*.bril --save
	turnt -e insn-counter -j $(nproc) lessons/2/test/*.bril --save
	turnt -e trivial-dce -e lvn -e lvn-dce -j $(nproc) lessons/3/test/*.bril --save