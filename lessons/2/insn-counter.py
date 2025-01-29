#!/usr/bin/env python3
import argparse
import subprocess
import json

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='BRIL Program Instruction Counter')
    parser.add_argument('--file', required=True, help='Path to the BRIL program')
    parser.add_argument('--output', help='Path to the output file')

    args = parser.parse_args()

    with open(args.file, 'r') as f:
        # if the file ends with .bril (instead of .json)
        # run bril2json on the file
        if args.file.endswith('.bril'):
            bril2json = subprocess.run(['bril2json'], stdin=f, stdout=subprocess.PIPE)
            data = bril2json.stdout.decode('utf-8')
            program = json.loads(data)
        else:
            program = json.load(f)

    # loop through all the functions in the program
    for func in program['functions']:
        new_insns = [
            {
                "dest": "__insn_count",
                "op": "const",
                "type": "int",
                "value": 0
            },
            {
                "dest": "__insn_one",
                "op": "const",
                "type": "int",
                "value": 1
            }
        ]
        for instr in func['instrs']:
            if "label" in instr:
                # If the instruction is a label, pass
                new_insns.append(instr)
                continue
            # Add an instruction to increment the instruction count
            new_insns.append({
                "dest": "__insn_count",
                "op": "add",
                "type": "int",
                "args": ["__insn_count", "__insn_one"]
            })

            if instr['op'] == 'ret':
                # If the instruction is a return instruction
                # Print the instruction count
                new_insns.append({
                    'op': 'print',
                    'args': ['__insn_count']
                })

            # Add the original instruction
            if instr['op'] != 'print':
                # Remove the print instructions
                # To avoid confusing prints
                new_insns.append(instr)
        # Also print at the very end of the function in case there is no return
        new_insns.append({
            'op': 'print',
            'args': ['__insn_count']
        })

        func['instrs'] = new_insns
            
                
    # Write the json to the output file
    if args.output:
        with open(args.output, 'w') as f:
            json.dump(program, f, indent=2)
    else:
        # Print the json to stdout
        print(json.dumps(program, indent=2))


