#!/usr/bin/env python3
import json
import sys

if __name__ == '__main__':
    # Implements dead code elimination
    
    # Read the program in from stdin as json
    program = json.load(sys.stdin)

    # loop through all the functions in the program
    for func in program['functions']:
        changed = True
        while changed:
            changed = False
            new_insns = []

            used_vars = set()
            for instr in func['instrs']:
                if "args" in instr:
                    for arg in instr['args']:
                        used_vars.add(arg)

            for instr in func['instrs']:
                if "dest" in instr and instr['dest'] not in used_vars:
                    changed = True
                    continue
                new_insns.append(instr)

            func['instrs'] = new_insns
            
    # Dump the program to stdout
    json.dump(program, sys.stdout, indent=2)


