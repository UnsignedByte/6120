#!/usr/bin/env python3
import json
import sys
import os

sys.path.append(os.path.join(os.path.dirname(__file__), "..", ".."))
from utils.passes import DataFlowPass, FunctionPass, BasicBlock


class ReachingDefs(DataFlowPass):
    def init(self):
        return set()

    def args(self):
        return set((a["name"], -1, -1) for a in self.func.get("args", []))

    def meet(self, inp):
        return set().union(*inp)

    def transfer(self, bidx: int):
        block = self.blocks[bidx]
        in_values = self.in_values[bidx]

        out_values = set(in_values)
        for i, instr in enumerate(block.instrs):
            if "dest" in instr:
                # Remove all definitions of this variable
                discard = set()
                for val in out_values:
                    if val[0] == instr["dest"]:
                        discard.add(val)

                out_values -= discard

                # Add this definition
                out_values.add((instr["dest"], bidx, i))

        return out_values

    def to_str(self, val: any):
        bindings = {}

        for var, bidx, i in val:
            # Figure out what value this is
            if bidx == -1:
                value = "?"
            else:
                inst = self.blocks[bidx].instrs[i]
                if inst.get("op", "") == "const":
                    value = str(inst["value"])
                else:
                    value = f"{inst['op']} {' '.join(inst.get('args', []))}"

            if not var in bindings:
                bindings[var] = set()

            bindings[var].add(value)

        output = []
        for var, values in bindings.items():
            if len(values) == 1:
                output.append(f"{var}={values.pop()}")
            else:
                # Sort the values for consistency
                values = sorted(list(values))
                output.append(f"{var}={{{', '.join(values)}}}")

        # Sort the output for consistency
        output.sort()
        return ", ".join(output)


if __name__ == "__main__":
    # Implements dead code elimination

    # Read the program in from stdin as json
    program = json.load(sys.stdin)

    for func in program["functions"]:
        pass_ = ReachingDefs(func)
        pass_.run()
        print(pass_)
