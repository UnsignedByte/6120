#!/usr/bin/env python3
import json
import os
import sys
sys.path.append(os.path.join(os.path.dirname(__file__), '..'))
from utils.passes import FunctionPass, BasicBlock

class Diagnostics(FunctionPass):
    def basic_block(self, block):
        print(block)
        return super().basic_block(block)

if __name__ == '__main__':
    # Prints the basic blocks of a function
    program = json.load(sys.stdin)

    for func in program['functions']:
        print(func['name'])
        pass_ = Diagnostics(func)
        pass_.run()




