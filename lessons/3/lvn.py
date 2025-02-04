#!/usr/bin/env python3
import json
import os
import sys
sys.path.append(os.path.join(os.path.dirname(__file__), '..', '..'))
from utils.passes import FunctionPass, BasicBlock

class LVNTable:
    def __init__(self):
        self.unique_instr_id = 0

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

    def __repr__(self):
        rep = ""
        for k, v in self.id2name.items():
            rep += f'{k}\t| {v}\t|\t{self.id2value[k]}\n'
        
        return rep

def last_write_map(block: BasicBlock) -> dict[str, int]:
    # Find the index of the last write of every variable 

    m = {}

    for i, instr in enumerate(block.instrs):
        if 'dest' in instr:
            m[instr['dest']] = i
    
    return m

def const_fold(table: LVNTable, instr):
    commutative_ops = {'add', 'mul', 'and', 'or'}
    op_map = {
        'add': lambda x, y: x + y,
        'sub': lambda x, y: x - y,
        'mul': lambda x, y: x * y,
        'div': lambda x, y: x // y,
        'and': lambda x, y: x and y,
        'or': lambda x, y: x or y,
        'not': lambda x: not x
    }

    op = instr['op']
    if op in commutative_ops:
        instr['args'].sort()
    args = instr['args']

    values = [table.id2value[arg] for arg in args]
    if op in op_map and all(v[0] == 'const' for v in values):
        values = [v[2] for v in values]
        return ("const", instr['type'], op_map[op](*values))
    else:
        return (op, *args)

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
        self.names.add(f'v{self.unique_nid}')
        return f'v{self.unique_nid}'

    def basic_block(self, block):
        # print(block)
        table = LVNTable()

        lw_map = last_write_map(block)

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
                # If this is a string, then we should check if its representative will be written later.
                # This is to deal with the case
                #   x = 0
                # NEW_BLOCK:
                #   y = x
                #   x = 1
                #   z = y
                rep = table.id2name[args[0]]
                if isinstance(value, str) and lw_map.get(value, 0) > i and rep == value:
                    new_name = self.construct_unique_name()
                    new_block.append(
                        {
                            "op": "id",
                            'dest': new_name,
                            'args': [rep]
                        }
                    )
                    table.id2name[args[0]] = new_name

            elif op == 'const':
                value = ("const", instr['type'], instr['value'])
            elif op in {'add', 'sub', 'mul', 'div','and', 'or', 'not'}:
                value = const_fold(table, instr)
            else:
                # This is an unknown instruction, treat it as something that
                # may have side effects
                value = ("unknown", table.unique_instr_id)
                table.unique_instr_id += 1
            # print(table)

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
                    if lw_map.get(instr['dest'], 0) > i:
                        instr['dest'] = self.construct_unique_name()
                        
                    old_name = name
                    name = instr['dest']
                    vid = table.add(name, value)

                    # Map the old name to the new id
                    table.name2id[old_name] = vid

                    # Recreate the instruction from the value
                    if isinstance(value, tuple):
                        if value[0] == 'const':
                            new_block.append({
                                **instr,
                                'op': 'const',
                                'dest': name,
                                'type': value[1],
                                'value': value[2]
                            })
                        elif value[0] == 'unknown':
                            if 'args' in instr:
                                instr['args'] = [table.id2name[arg] for arg in instr['args']]
                            new_block.append({
                                **instr,
                                'dest': name
                            })
                        else:
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




