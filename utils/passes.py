from abc import ABC, abstractmethod


# A BRIL basic block
class BasicBlock:

    def __init__(self, name, instrs):
        self.name = name
        self.instrs = instrs

    def flatten(self):
        if self.name is not None:
            return [{"label": self.name}, *self.instrs]
        return self.instrs

    def __repr__(self):
        return f"{self.name}: {self.instrs}"

    def __len__(self):
        return len(self.instrs) + (1 if self.name is not None else 0)


def generate_basic_blocks(func) -> list[BasicBlock]:
    # Split the function into basic blocks
    blocks = []
    current_block = []
    name = None
    for instr in func["instrs"]:
        if "label" in instr:  # Header of a new block
            if len(current_block) > 0 or name is not None:
                blocks.append(BasicBlock(name, current_block))

            name = instr["label"]
            current_block = []
        else:
            current_block.append(instr)

        # If the instruction is a control flow instruction
        # I.E. a br, jmp, or ret instruction
        if instr.get("op", "") in {"br", "jmp", "ret"}:
            if len(current_block) > 0 or name is not None:
                blocks.append(BasicBlock(name, current_block))
            name = None
            current_block = []

    if len(current_block) > 0 or name is not None:
        blocks.append(BasicBlock(name, current_block))
    return blocks


def generate_cfg(blocks):
    succs = [[] for _ in blocks]

    blocks = list(enumerate(blocks))

    name_map = {block.name: idx for (idx, block) in blocks}

    for i, block in blocks:
        if len(block.instrs) == 0:
            if i == len(blocks) - 1:
                succs[i] = []
            else:
                succs[i] = [i + 1]
        else:
            last_instr = block.instrs[-1]
            # If this is a br, jmp, or ret instruction
            if last_instr.get("op", "") in {"jmp", "br"}:
                # There should be two successors
                succs[i] = [name_map[s] for s in last_instr["labels"]]
            elif last_instr.get("op", "") == "ret" or i == len(blocks) - 1:
                succs[i] = []
            else:
                succs[i] = [i + 1]

    # Now create the predecessors set
    preds = [[] for _ in blocks]
    for i, block in blocks:
        for succ in succs[i]:
            preds[succ].append(i)

    return preds, succs


# Abstract base class for all function passes
# Contains some extra functionality for modifying individual basic blocks
class FunctionPass(ABC):
    def __init__(self, func):
        self.func = func
        self.blocks = generate_basic_blocks(func)

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

        # Loop through each basic block and run the basic_block method
        new_blocks = list(map(self.basic_block, self.blocks))

        self.blocks = new_blocks

        # Reassemble the function
        new_instrs = map(lambda block: block.flatten(), new_blocks)
        self.func["instrs"] = [instr for block in new_instrs for instr in block]

        self.after()

    def get(self):
        return self.func


class DataFlowPass(FunctionPass):
    def __init__(self, func, reverse: bool = False):
        super().__init__(func)
        self.blocks = generate_basic_blocks(func)
        self.in_values = [self.init() for _ in self.blocks]
        self.out_values = [self.init() for _ in self.blocks]
        self.reverse = reverse

    @abstractmethod
    def to_str(self, val: any):
        # Method to convert a value to a string
        pass

    @abstractmethod
    def init(self):
        # Method to initialize the out values for all blocks
        pass

    @abstractmethod
    def args(self):
        # Method to initialize the in values for the entry block
        pass

    @abstractmethod
    def meet(self, inp: list[any]):
        pass

    @abstractmethod
    def transfer(self, bidx: int):
        pass

    def run(self):
        """
        Follow this Pseudocode:

        in[entry] = init1
        out[*] = init2

        worklist = all blocks
        while worklist is not empty:
            b = pick any block from worklist
            in[b] = merge(out[p] for every predeccessor p of b)
            out[b] = transfer(b, in[b])
            if out[b] changed:
                worklist += successors of b
        """

        # First, we need to generate sets of predecessors and successors for each block

        preds, succs = generate_cfg(self.blocks)

        if self.reverse:
            preds, succs = succs, preds

        worklist = list(range(len(self.blocks)))

        def is_entry(bidx):
            if not self.reverse:
                return bidx == 0
            else:
                return len(preds[bidx]) == 0

        while len(worklist) > 0:
            b = worklist.pop(0)

            inputs = [self.out_values[p] for p in preds[b]]
            if is_entry(b):
                inputs.append(self.args())
            self.in_values[b] = self.meet(inputs)

            new_out = self.transfer(b)
            if new_out != self.out_values[b]:
                self.out_values[b] = new_out
                worklist += succs[b]

        # Now that we have the final values, we can apply them to the function
        super().run()

    def __repr__(self):
        output = ""
        # Print output information
        output += f"{self.func['name']} {{\n"
        for i, block in enumerate(self.blocks):
            name = block.name if block.name else "unknown"
            output += f".{name}:\n"
            # output the input and output values for this block
            output += f"\tin: {self.to_str(self.in_values[i])}\n"
            output += f"\tout: {self.to_str(self.out_values[i])}\n"
        output += "}\n"
        return output
