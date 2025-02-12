#!/usr/bin/env python3
import json
import sys
import os

sys.path.append(os.path.join(os.path.dirname(__file__), "..", ".."))
from utils.passes import DataFlowPass


class LiveVars(DataFlowPass):
    def __init__(self, func):
        super().__init__(func, True)

    def init(self):
        return set()

    def args(self):
        return self.init()

    def meet(self, inp):
        return set().union(*inp)

    def transfer(self, bidx: int):
        block = self.blocks[bidx]
        in_values = self.in_values[bidx]

        out_values = set(in_values)
        for i, instr in reversed(list(enumerate(block.instrs))):
            # Remove the defined variable from the out set
            if "dest" in instr:
                out_values.discard(instr["dest"])

            # Add the used variables to the out set
            for arg in instr.get("args", []):
                out_values.add(arg)

        return out_values

    def to_str(self, val: any):
        return ", ".join(sorted(list(val)))


if __name__ == "__main__":
    # Implements dead code elimination

    # Read the program in from stdin as json
    program = json.load(sys.stdin)

    for func in program["functions"]:
        pass_ = LiveVars(func)
        pass_.run()
        print(pass_)
