from abc import ABC, abstractmethod

# A BRIL basic block
class BasicBlock:
  def __init__(self, name, instrs):
    self.name = name
    self.instrs = instrs

  def flatten(self):
    if self.name is not None:
      return [
        {'label': self.name},
        *self.instrs
      ]
    return self.instrs

  def __repr__(self):
    return f"{self.name}: {self.instrs}"


# Abstract base class for all function passes
# Contains some extra functionality for modifying individual basic blocks
class FunctionPass(ABC):
  def __init__(self, func):
    self.func = func

  def get_basic_blocks(self) -> list[BasicBlock]:
    # Split the function into basic blocks
    blocks = []
    current_block = []
    name = None
    for instr in self.func['instrs']:
      if 'label' in instr: # Header of a new block
        if len(current_block) > 0:
          blocks.append(BasicBlock(name, current_block))

        name = instr['label']
        current_block = []
      else:
        current_block.append(instr)

      # If the instruction is a control flow instruction
      # I.E. a br, jmp, or ret instruction
      if instr.get('op', '') in {'br', 'jmp', 'ret'}:
        if len(current_block) > 0:
          blocks.append(BasicBlock(name, current_block))
        name = None
        current_block = []
      
    if len(current_block) > 0:
      blocks.append(BasicBlock(name, current_block))
    return blocks
  
  def basic_block(self, block: BasicBlock) -> BasicBlock:
    # Modify the basic block
    return block

  def before(self):
    # Run before the pass begins
    pass

  def after(self):
    # Run after the pass ends
    pass

  def run(self):
    self.before()

    # First, generate the basic blocks for the function
    blocks = self.get_basic_blocks()

    # Loop through each basic block and run the basic_block method
    new_blocks = map(self.basic_block, blocks)

    # Reassemble the function
    new_instrs = map(lambda block: block.flatten(), new_blocks)

    self.func['instrs'] = [instr for block in new_instrs for instr in block]

    self.after()

  def get(self):
    return self.func

