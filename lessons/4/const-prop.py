#!/usr/bin/env python3
import json
import sys
import os

sys.path.append(os.path.join(os.path.dirname(__file__), "..", ".."))
from utils.passes import DataFlowPass


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
            else:
                ret[key] = None

        return ret

    def transfer(self, bidx: int):
        block = self.blocks[bidx]
        in_values = self.in_values[bidx]

        # Operations that can be constant propagated
        op_map = {
            "add": lambda x, y: ("int", x + y),
            "sub": lambda x, y: ("int", x - y),
            "mul": lambda x, y: ("int", x * y),
            "div": lambda x, y: ("int", x // y),
            "eq": lambda x, y: ("bool", x == y),
            "lt": lambda x, y: ("bool", x < y),
            "gt": lambda x, y: ("bool", x > y),
            "le": lambda x, y: ("bool", x <= y),
            "ge": lambda x, y: ("bool", x >= y),
            "and": lambda x, y: ("bool", x and y),
            "or": lambda x, y: ("bool", x or y),
            "not": lambda x: ("bool", not x),
            "fadd": lambda x, y: ("float", x + y),
            "fsub": lambda x, y: ("float", x - y),
            "fmul": lambda x, y: ("float", x * y),
            "fdiv": lambda x, y: ("float", x / y),
            "feq": lambda x, y: ("bool", x == y),
            "flt": lambda x, y: ("bool", x < y),
            "fgt": lambda x, y: ("bool", x > y),
            "fle": lambda x, y: ("bool", x <= y),
            "fge": lambda x, y: ("bool", x >= y),
        }

        out_values = {**in_values}
        for i, instr in enumerate(block.instrs):
            args = [out_values.get(arg, None) for arg in instr.get("args", [])]
            if "dest" in instr:
                if instr.get("op", "") == "const":
                    # If the instruction is a constant, then it is constant propagatable
                    out_values[instr["dest"]] = (instr["type"], instr["value"])
                elif instr.get("op", "") in op_map and all(
                    arg is not None for arg in args
                ):
                    args = [v[1] for v in args]
                    # If all the arguments are constant propagatable, then this instruction is constant propagatable
                    out_values[instr["dest"]] = op_map[instr["op"]](*args)
                else:
                    # Otherwise, remove the variable from the out set
                    out_values[instr["dest"]] = None
        return out_values

    def before(self):
        def vals_str(vals):
            vals = [f"{k}: {v[0]} = {v[1]}" for k, v in vals.items() if v is not None]
            return ", ".join(sorted(vals))

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
