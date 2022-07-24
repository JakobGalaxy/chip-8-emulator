/// **NOTE:** the stack is only used for storing return addresses when calling subroutines
pub struct Stack {
    // 48 bytes of stack memory (24 x 2 bytes)
    pub memory: [u16; 0x18],

    pub stack_pointer: u16,
}

impl Stack {
    pub fn new() -> Stack {
        return Stack {
            memory: [0; 0x18],
            stack_pointer: 0,
        };
    }

    pub fn pop(&mut self) -> u16 {
        if self.stack_pointer <= 0 {
            panic!("stack underflow!");
        }

        self.stack_pointer -= 1;
        return self.memory[self.stack_pointer as usize];
    }

    pub fn push(&mut self, return_address: u16) {
        if (self.stack_pointer as usize) >= self.memory.len() {
            panic!("stack overflow!");
        }

        self.memory[self.stack_pointer as usize] = return_address;
        self.stack_pointer += 1;
    }
}