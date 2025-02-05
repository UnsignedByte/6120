#!/usr/bin/env python3
import json
import sys
import os

sys.path.append(os.path.join(os.path.dirname(__file__), "..", ".."))
from utils.passes import FunctionPass, BasicBlock


class LocalDCE(FunctionPass):
    def basic_block(self, block):
        written_not_read = set()

        new_block = []

        # Go in reverse because we want to discard writes that occur without other writes after them
        for instr in block.instrs[::-1]:
            if "args" in instr:
                for arg in instr["args"]:
                    written_not_read.discard(arg)

            dead = False
            if "dest" in instr:
                # If this value is written but not read, we can remove it
                if instr["dest"] in written_not_read:
                    dead = True
                written_not_read.add(instr["dest"])

            if not dead:
                new_block.append(instr)

        return BasicBlock(block.name, new_block[::-1])


if __name__ == "__main__":
    # Implements dead code elimination

    # Read the program in from stdin as json
    program = json.load(sys.stdin)

    # GLOBAL DCE
    # loop through all the functions in the program
    for func in program["functions"]:
        changed = True
        while changed:
            changed = False
            new_insns = []

            used_vars = set()
            for instr in func["instrs"]:
                if "args" in instr:
                    for arg in instr["args"]:
                        used_vars.add(arg)

            for instr in func["instrs"]:
                if "dest" in instr and instr["dest"] not in used_vars:
                    changed = True
                    continue
                new_insns.append(instr)

            func["instrs"] = new_insns

    # LOCAL DCE
    new_funcs = []
    for func in program["functions"]:
        pass_ = LocalDCE(func)
        pass_.run()
        new_funcs.append(func)

    program["functions"] = new_funcs

    # Dump the program to stdout
    json.dump(program, sys.stdout, indent=2)
