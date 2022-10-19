use generational_arena::Arena;
use generational_arena::Index;

pub const BLOCK_SIZE: usize = 64;

pub type Block = [f32; BLOCK_SIZE];

pub type InputIndex = Index;
pub type OutputIndex = Index;

pub const SILENCE: Block = [0f32; BLOCK_SIZE];

pub struct SynthContext {
    blocks: Arena<Block>,
}

impl SynthContext {
    pub fn new() -> Self {
        let initial_blocks = 1024;

        Self {
            blocks: Arena::with_capacity(initial_blocks),
        }
    }

    pub fn new_block(&mut self) -> Index {
        self.blocks.insert(SILENCE)
    }

    pub fn block(&self, index: Index) -> Option<&Block> {
        self.blocks.get(index)
    }

    pub fn block_mut(&mut self, index: Index) -> Option<&mut Block> {
        self.blocks.get_mut(index)
    }
}

pub trait Operator {
    fn operate(&self, context: &mut SynthContext);
}

#[derive(Debug, Clone, Copy)]
pub struct SineSource {
    block: Index,
    frequency: f32,
    phase: f32,
}

impl SineSource {
    pub fn new(frequency: f32, context: &mut SynthContext) -> Self {
        Self {
            block: context.new_block(),
            frequency,
            phase: 0.0,
        }
    }
}
