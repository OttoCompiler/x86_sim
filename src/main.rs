const MEM_SIZE: usize = 0x10000; //     64 kilobytes
const STACK_START: u16 = 0xFFF0;


#[derive(Default, Debug)]
struct Registers {
    ax: u16, bx: u16, cx: u16, dx: u16,
    si: u16, di: u16, sp: u16, bp: u16,
    ip: u16,
    flags: u16, // [ ...|O|D|I|T|S|Z|A|P|C ]
}

struct X86Cpu {
    regs: Registers,
    memory: Box<[u8; MEM_SIZE]>,
    halted: bool,
}

impl X86Cpu {
    fn new() -> Self {
        let mut cpu = X86Cpu {
            regs: Registers::default(),
            memory: Box::new([0; MEM_SIZE]),
            halted: false,
        };
        cpu.regs.sp = STACK_START;
        cpu
    }
    fn fetch_u8(&mut self) -> u8 {
        let val = self.memory[self.regs.ip as usize];
        self.regs.ip += 1;
        val
    }

    fn fetch_u16(&mut self) -> u16 {
        let low = self.fetch_u8() as u16;
        let high = self.fetch_u8() as u16;
        (high << 8) | low
    }

    fn push(&mut self, val: u16) {      //for stack
        self.regs.sp -= 2;
        let addr = self.regs.sp as usize;
        self.memory[addr] = (val & 0xFF) as u8;
        self.memory[addr + 1] = (val >> 8) as u8;
    }

    fn pop(&mut self) -> u16 {
        let addr = self.regs.sp as usize;
        let val = ((self.memory[addr + 1] as u16) << 8) | (self.memory[addr] as u16);
        self.regs.sp += 2;
        val
    }

    fn set_sz(&mut self, val: u16) {
        if val == 0 {
            self.regs.flags |= 0x40;
        } else {
            self.regs.flags &= !0x40;
        }
        if val & 0x8000 != 0 {
            self.regs.flags |= 0x80;
        } else {
            self.regs.flags &= !0x80;
        }
    }

    fn step(&mut self) {
        if self.halted { return; }
        let opcode = self.fetch_u8();
        match opcode {
            // MOV reg imm16
            0xB8 => { self.regs.ax = self.fetch_u16(); }
            0xBB => { self.regs.bx = self.fetch_u16(); }
            0xB9 => { self.regs.cx = self.fetch_u16(); }
            0x40 => { self.regs.ax = self.regs.ax.wrapping_add(1); self.set_sz(self.regs.ax); }
            0x48 => { self.regs.ax = self.regs.ax.wrapping_sub(1); self.set_sz(self.regs.ax); }
            0x49 => { self.regs.cx = self.regs.cx.wrapping_sub(1); self.set_sz(self.regs.cx); }
            0x50 => { let v = self.regs.ax; self.push(v); }
            0x58 => { self.regs.ax = self.pop(); }
            0xF7 => {
                let next = self.fetch_u8();
                if next == 0xE1 {
                    let res = (self.regs.ax as u32) * (self.regs.cx as u32);
                    self.regs.ax = res as u16;
                    self.regs.dx = (res >> 16) as u16;
                }
            }

            // CMP CX
            0x81 => {
                let next = self.fetch_u8();
                if next == 0xF9 {
                    let imm = self.fetch_u16();
                    let res = self.regs.cx.wrapping_sub(imm);
                    self.set_sz(res);
                }
            }

            // JNZ
            0x75 => {
                let offset = self.fetch_u8() as i8;
                if (self.regs.flags & 0x40) == 0 {
                    self.regs.ip = (self.regs.ip as i16 + offset as i16) as u16;
                }
            }

            // HLT
            0xF4 => { self.halted = true; }

            _ => {
                println!("Unknown opcode: 0x{:02X} at IP: 0x{:04X}", opcode, self.regs.ip.wrapping_sub(1));
                self.halted = true;
            }
        }
    }
}


fn load_factorial_program(cpu: &mut X86Cpu) {
    let program: Vec<u8> = vec![
        0xB8, 0x01, 0x00,
        0xB9, 0x05, 0x00,
        0xF7, 0xE1,
        0x49,
        0x81, 0xF9, 0x01, 0x00,
        0x75, 0xF7,
        0x50,
        0xF4
    ];

    for (i, &byte) in program.iter().enumerate() {
        cpu.memory[i] = byte;
    }
}


fn main() {
    println!("--- x86 Real Mode Simulator ---");
    let mut cpu = X86Cpu::new();
    load_factorial_program(&mut cpu);
    println!("Calculating 5! (Factorial of 5)...");
    let mut steps = 0;

    while !cpu.halted && steps < 50 {
        cpu.step();
        steps += 1;
        println!("Step {:02} | IP: 0x{:04X} | AX: {:5} | CX: {:5}",
                 steps, cpu.regs.ip, cpu.regs.ax, cpu.regs.cx);
    }

    let result = cpu.pop();
    println!("\nSimulation Halted.");
    println!("Final Factorial Result on Stack: {}", result);

}