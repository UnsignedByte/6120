#!/usr/bin/env python3
import json
import sys
import os

sys.path.append(os.path.join(os.path.dirname(__file__), "..", ".."))
from utils.passes import DataFlowPass


class AvailableExpr(DataFlowPass):
    def init(self):
        return None

    def args(self):
        return set()

    def meet(self, inp):
        # Filter out nones
        inp = [x for x in inp if x is not None]
        if not inp:
            return None

        return inp[0].intersection(*inp[1:])

    def transfer(self, bidx: int):
        block = self.blocks[bidx]
        in_values = self.in_values[bidx]

        # Operations that do not have side effects
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
            "and",
            "or",
            "not",
            "fadd",
            "fsub",
            "fmul",
            "fdiv",
            "feq",
            "flt",
            "fgt",
            "fle",
            "fge",
            "id",
        }

        out_values = set(in_values)
        for instr in block.instrs:
            if instr.get("op", None) == "const":
                out_values.add(("const", instr["value"]))
            elif instr.get("op", None) in const_ops:
                out_values.add((instr["op"], *instr.get("args", [])))

            if "dest" in instr:
                # Remove all expressions that read from this variable
                discard = set()
                for val in out_values:
                    if val[0] != "const":
                        used_args = set(val[1:])
                        if instr["dest"] in used_args:
                            discard.add(val)

                out_values -= discard

        return out_values

    def before(self):
        def vals_str(vals):
            vals = [f"({' '.join(map(str, v))})" for v in vals]
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
        pass_ = AvailableExpr(func)
        pass_.run()
