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

    def before(self):
        def vals_str(vals):
            return ", ".join(sorted(list(vals)))

        # Print output information
        for i, block in enumerate(self.blocks):
            name = block.name if block.name else "unknown"
            print(f".{name}:")
            # Print the input and output values for this block
            print(f"\tout: {vals_str(self.out_values[i])}")
            print(f"\tin: {vals_str(self.in_values[i])}")


if __name__ == "__main__":
    # Implements dead code elimination

    # Read the program in from stdin as json
    program = json.load(sys.stdin)

    for func in program["functions"]:
        pass_ = LiveVars(func)
        pass_.run()
