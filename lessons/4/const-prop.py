#!/usr/bin/env python3
import json
import sys
import os

sys.path.append(os.path.join(os.path.dirname(__file__), "..", ".."))
from utils.passes import DataFlowPass, FunctionPass, BasicBlock


class ConstProp(DataFlowPass):
    def init(self):
        return {}

    def args(self):
        return self.init()

    def meet(self, inp):
        ret = {}
        # Keep only the values that are the same in all the inputs
        keys = set().union(*[x.keys() for x in inp])
        for key in keys:
            values = set(x[key] for x in inp if key in x)
            if len(values) == 1:
                ret[key] = values.pop()

        return ret

    def transfer(self, bidx: int):
        block = self.blocks[bidx]
        in_values = self.in_values[bidx]

        # Operations that can be constant propagated
        const_ops = {
            "add",
            "sub",
            "mul",
            "div",
            "eq",
            "lt",
            "gt",
            "le",
            "ge",
            "not",
            "and",
            "or",
            "fadd",
            "fsub",
            "fmul",
            "fdiv",
            "feq",
            "flt",
            "fgt",
            "fle",
            "fge",
            "const",
        }

        out_values = {**in_values}
        for i, instr in enumerate(block.instrs):
            if "dest" in instr:
                if instr.get("op", "") in const_ops and all(
                    arg in out_values for arg in instr.get("args", [])
                ):
                    # If all the arguments are constant propagatable, then this instruction is constant propagatable
                    out_values[instr["dest"]] = bidx
                else:
                    # Otherwise, remove the variable from the out set
                    if instr["dest"] in out_values:
                        del out_values[instr["dest"]]
        return out_values

    def before(self):
        def vals_str(vals):
            return ", ".join(sorted(list(vals)))

        # Print output information
        for i, block in enumerate(self.blocks):
            name = block.name if block.name else "unknown"
            print(f".{name}:")
            # Print the input and output values for this block
            print(f"\tin: {vals_str(self.in_values[i])}")
            print(f"\tout: {vals_str(self.out_values[i])}")


if __name__ == "__main__":
    # Implements dead code elimination

    # Read the program in from stdin as json
    program = json.load(sys.stdin)

    for func in program["functions"]:
        pass_ = ConstProp(func)
        pass_.run()
