#!/usr/bin/env python3
import json
import sys
import os
import math

sys.path.append(os.path.join(os.path.dirname(__file__), "..", ".."))
from utils.passes import DataFlowPass


class IntervalPass(DataFlowPass):
    defaults = {
        "int": (-math.inf, math.inf),
        "float": (-math.inf, math.inf),
        "bool": (0, 1),
    }

    def __init__(self, func, max_iters=100):
        super().__init__(func)
        self.max_iterations = max_iters

    def init(self):
        return (0, {})

    def args(self):
        # All arguments have interval
        return (
            0,
            {
                a["name"]: IntervalPass.defaults[a["type"]]
                for a in self.func.get("args", [])
            },
        )

    def meet(self, inp):
        ret = {}

        itx = max(x for (x, _) in inp)

        inp = [x[1] for x in inp]

        # Get the largest interval given the intervals of the inputs
        keys = set().union(*[x.keys() for x in inp])
        for key in keys:
            intervals = [x[key] for x in inp if key in x]
            min_ = min(i[0] for i in intervals)
            max_ = max(i[1] for i in intervals)

            ret[key] = (min_, max_)

        return (itx, ret)

    def transfer(self, bidx: int):
        block = self.blocks[bidx]
        itx, in_values = self.in_values[bidx]

        def add(x, y):
            return (x[0] + y[0], x[1] + y[1])

        def sub(x, y):
            return (x[0] - y[1], x[1] - y[0])

        def mk_binop(op):
            def binop(x, y):
                vals = [op(x[0], y[0]), op(x[0], y[1]), op(x[1], y[0]), op(x[1], y[1])]

                return (min(vals), max(vals))

            return binop

        # Operations that can be constant propagated
        op_map = {
            "add": add,
            "sub": sub,
            "mul": mk_binop(lambda a, b: a * b),
            "div": mk_binop(lambda a, b: a // b),
            "eq": mk_binop(lambda a, b: a == b),
            "lt": mk_binop(lambda a, b: a < b),
            "gt": mk_binop(lambda a, b: a > b),
            "le": mk_binop(lambda a, b: a <= b),
            "ge": mk_binop(lambda a, b: a >= b),
            "and": mk_binop(lambda a, b: a and b),
            "or": mk_binop(lambda a, b: a or b),
            "not": lambda x: (not x[1], not x[0]),
            "fadd": add,
            "fsub": sub,
            "fmul": mk_binop(lambda a, b: a * b),
            "fdiv": mk_binop(lambda a, b: a / b),
            "feq": mk_binop(lambda a, b: a == b),
            "flt": mk_binop(lambda a, b: a < b),
            "fgt": mk_binop(lambda a, b: a > b),
            "fle": mk_binop(lambda a, b: a <= b),
            "fge": mk_binop(lambda a, b: a >= b),
        }

        out_values = {**in_values}
        for instr in block.instrs:
            args = [
                out_values.get(arg, (-math.inf, math.inf))
                for arg in instr.get("args", [])
            ]
            if "dest" in instr:
                dest = instr["dest"]
                if instr.get("op", "") == "const":
                    # If the instruction is a constant, then it is constant propagatable
                    out_values[dest] = (instr["value"], instr["value"])
                elif instr.get("op", "") == "id":
                    out_values[dest] = args[0]
                elif instr.get("op", "") in op_map:
                    # If all the arguments are constant propagatable, then this instruction is constant propagatable
                    out_values[dest] = op_map[instr["op"]](*args)
                else:
                    # Otherwise, the value is unknown
                    out_values[dest] = IntervalPass.defaults[instr["type"]]

        # If the iterations is at the max number, then anything changed in this block will become unknown
        if itx >= self.max_iterations:
            for k, v in out_values.items():
                o_v = in_values.get(k, IntervalPass.defaults["int"])

                # First, try to merge the two intervals to see if they are contained in each
                # other. If they are, then the value is the same
                v = (min(v[0], o_v[0]), max(v[1], o_v[1]))

                n_v = (
                    v[0] if v[0] == o_v[0] else -math.inf,
                    v[1] if v[1] == o_v[1] else math.inf,
                )

                out_values[k] = n_v

            return (self.max_iterations, out_values)

        return (itx + 1, out_values)

    def to_str(self, val):
        itx, vals = val

        ret = []
        for k, v in vals.items():
            ret.append(f"{k} = [{v[0]}, {v[1]}]")
        return ", ".join(sorted(ret))


if __name__ == "__main__":
    # Implements dead code elimination

    # Read the program in from stdin as json
    program = json.load(sys.stdin)

    for func in program["functions"]:
        pass_ = IntervalPass(func)
        pass_.run()
        print(pass_)
