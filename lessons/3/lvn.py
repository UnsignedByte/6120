#!/usr/bin/env python3
import json
import os
import sys
sys.path.append(os.path.join(os.path.dirname(__file__), '..', '..'))
from utils.passes import FunctionPass, BasicBlock

class LVNTable:
    def __init__(self):
        # Maps IDs to names in the table
        self.id2name = {}
        # Maps names to IDs in the table
        self.name2id = {}
        # Value table
        self.value2id = {}
        self.id2value = {}

    def add(self, name, value):
        if value in self.value2id:
            vid = self.value2id[value]
            self.name2id[name] = vid
        else:
            # A new value
            vid = len(self.id2name)
            self.id2name[vid] = name
            self.id2value[vid] = value
            self.name2id[name] = vid
            self.value2id[value] = vid
        return vid
    
    def canonicalize(self, instr):
        if 'args' in instr:
            instr['args'] = [
                self.get_representative(self.name_map[arg])
                for arg in instr['args']
            ]
    
    def get_representative(self, value_id: int) -> str:
        return self.id2name[value_id]

class LocalValueNumbering(FunctionPass):
    def before(self):
        # Collect all the names in the function
        self.names = set()
        self.unique_nid = 0
        if "args" in self.func:
            self.names.update([arg['name'] for arg in self.func['args']])
        for instr in self.func['instrs']:
            if 'dest' in instr:
                self.names.add(instr['dest'])
            if 'args' in instr:
                self.names.update(instr['args'])
        
    def construct_unique_name(self):
        while f'v{self.unique_nid}' in self.names:
            self.unique_nid += 1
        return f'v{self.unique_nid}'

    def basic_block(self, block):
        # print(block)
        table = LVNTable()

        new_block = []
        for i in range(len(block.instrs)):
            instr = block.instrs[i]

            # Rewrite the arguments of the instruction
            if 'args' in instr:
                new_args = []
                for arg in instr['args']:
                    if arg in table.name2id:
                        new_args.append(table.name2id[arg])
                    else:
                        # This is a new value that has not been written in this basic block
                        new_args.append(table.add(arg, arg))
                instr['args'] = new_args

            op = instr['op']
            # Convert the instruction to a value
            value = None
            if op == 'id':
                args = instr['args']
                assert len(args) == 1
                # Find the value in the table
                value = table.id2value[args[0]]
            elif op == 'const':
                value = instr['value']
            elif op in {'add', 'sub', 'mul', 'div'}:
                args = instr['args']
                assert len(args) == 2
                # If both argument values are constants, we can evaluate the operation
                values = [table.id2value[arg] for arg in args]
                if all(isinstance(v, int) for v in values):
                    if op == 'add':
                        value = values[0] + values[1]
                    elif op == 'sub':
                        value = values[0] - values[1]
                    elif op == 'mul':
                        value = values[0] * values[1]
                    elif op == 'div':
                        value = values[0] // values[1]
                else:
                    # Canonicalize the arguments
                    args.sort()
                    # Construct the value number
                    value = (op, *args)
            elif op in {'and', 'or', 'not'}:
                args = instr['args']
                assert len(args) <= 2
                
                values = [table.id2value[arg] for arg in args]
                if all(isinstance(v, bool) for v in values):
                    if op == 'and':
                        value = values[0] and values[1]
                    elif op == 'or':
                        value = values[0] or values[1]
                    elif op == 'not':
                        value = not values[0]
                else:
                    args.sort()
                    value = (op, *args)

            if 'dest' in instr:
                name = instr['dest']
                if value in table.value2id:
                    # This is an old value
                    vid = table.add(name, value)
                    rep = table.get_representative(vid)

                    # Create an id instruction
                    new_block.append({
                        **instr,
                        'op': 'id',
                        'dest': name,
                        'args': [rep]
                    })
                else:
                    # This is a new value

                    # Check if the name will be rewritten later.
                    # If so, assign it a new name
                    for j in range(i + 1, len(block.instrs)):
                        if block.instrs[j].get('dest', '') == instr['dest']:
                            instr['dest'] = self.construct_unique_name()
                            break
                    old_name = name
                    name = instr['dest']
                    vid = table.add(name, value)

                    # Map the old name to the new id
                    table.name2id[old_name] = vid

                    # Recreate the instruction from the value
                    if value is None:
                        if 'args' in instr:
                            instr['args'] = [table.id2name[arg] for arg in instr['args']]
                        new_block.append(instr)
                    elif isinstance(value, int):
                        if 'args' in instr:
                            del instr['args']
                        new_block.append({
                            **instr,
                            'op': 'const',
                            'dest': name,
                            'value': value
                        })
                    elif isinstance(value, tuple):
                        new_block.append({
                            **instr,
                            'op': value[0],
                            'dest': name,
                            'args': [table.id2name[arg] for arg in value[1:]]
                        })
                    else:
                        raise ValueError(f'Unknown value type: {value}')
                    
            else:
                # Just reconstruct the instruction
                if 'args' in instr:
                    instr['args'] = [table.id2name[arg] for arg in instr['args']]
                new_block.append(instr)


        return BasicBlock(block.name, new_block)


if __name__ == '__main__':
    # Prints the basic blocks of a function
    program = json.load(sys.stdin)

    new_funcs = []

    for func in program['functions']:
        pass_ = LocalValueNumbering(func)
        pass_.run()
        new_funcs.append(func)
    
    program['functions'] = new_funcs

    json.dump(program, sys.stdout, indent=2)




