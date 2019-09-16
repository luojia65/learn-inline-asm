use std::borrow::Cow;
use std::mem::transmute;

const OPCODE_LOAD: u8 =     0b000_0011; 
const OPCODE_OP_IMM: u8 =   0b001_0011; 
const OPCODE_AUIPC: u8 =    0b001_0111; 
const OPCODE_STORE: u8 =    0b010_0011; 
const OPCODE_OP: u8 =       0b011_0011; 
const OPCODE_LUI: u8 =      0b011_0111; 
const OPCODE_BRANCH: u8 =   0b110_0011; 
const OPCODE_JALR: u8 =     0b110_0111; 
const OPCODE_JAL: u8 =      0b110_1111;

const FUNCT3_OP_ADD_SUB: u8 = 0b000;
const FUNCT3_OP_SLL: u8   = 0b001;
const FUNCT3_OP_SLT: u8   = 0b010;
const FUNCT3_OP_SLTU: u8  = 0b011;
const FUNCT3_OP_XOR: u8   = 0b100;
const FUNCT3_OP_SRL_SRA: u8 = 0b101;
const FUNCT3_OP_OR: u8    = 0b110;
const FUNCT3_OP_AND: u8   = 0b111;

#[derive(Debug)]
pub struct AddrSpace<'a> {
    linear: Vec<(u32, Cow<'a, [u8]>)>
}

impl<'a> AddrSpace<'a> {
    pub fn new() -> Self {
        Self { linear: Vec::new() }
    }

    pub fn insert_new(&mut self, base: u32, len: u32) {
        self.insert(base, vec![0u8; len as usize])
    }

    pub fn insert<D: Into<Cow<'a, [u8]>>>(&mut self, base: u32, data: D) {
        self.linear.push((base, data.into()));
        let mut cur = self.linear.len() - 1;
        while cur > 0 {
            if self.linear[cur].0 < self.linear[cur - 1].0 {
                self.linear.swap(cur, cur - 1)
            } else {
                break;
            }
            cur -= 1;
        }
    }

    pub fn get(&self, addr: u32) -> Option<u8> {
        let index = match self.linear.binary_search_by(|probe| probe.0.cmp(&addr)) {
            Ok(index) => index,
            Err(next) => next - 1,
        };
        let (base, elem) = &self.linear[index];
        if addr - base < elem.len() as u32 {
            Some(elem[(addr - base) as usize])
        } else {
            None
        }
    }

    pub fn set(&mut self, addr: u32, value: u8) {
        let index = match self.linear.binary_search_by(|probe| probe.0.cmp(&addr)) {
            Ok(index) => index,
            Err(next) => next - 1,
        };
        let (base, elem) = &mut self.linear[index];
        let index = (addr - *base) as usize;
        match elem {
            Cow::Borrowed(s) => {
                let mut v = s.to_vec();
                v[index] = value;
                *elem = Cow::Owned(v);
            },
            Cow::Owned(v) => v[index] = value,
        }
    }
}

#[derive(Debug)]
pub struct IntReg {
    x: [u32; 32],
}

impl IntReg {
    pub fn new() -> Self {
        Self { x: [0u32; 32] }
    }

    pub fn get(&self, i: u8) -> u32 {
        self.x[i as usize]
    }

    pub fn set(&mut self, i: u8, v: u32) {
        if i != 0 {
            self.x[i as usize] = v
        }
    }
}

fn main() {
    let prog: &[u8] = &[
        0x13, 0x04, 0x01, 0x01,
        0x13, 0x01, 0x01, 0xff,
        0x13, 0x37, 0x17, 0x00,
        0x13, 0x27, 0x07, 0x7d,
        0x13, 0x77, 0x27, 0x83,
        0x97, 0x00, 0x00, 0x00,
        0x6f, 0x01, 0x00, 0x00,
    ];
    let mut space = AddrSpace::new();
    space.insert(0, prog);
    let mut ints = IntReg::new();
    ints.set(2, 1000);
    let mut pc = 0;

    for _ in 0..=6 {
        let a = space.get(pc).unwrap(); 
        pc += 1;
        let b = space.get(pc).unwrap();
        pc += 1;
        let c = space.get(pc).unwrap();
        pc += 1;
        let d = space.get(pc).unwrap();
        pc += 1;
        let opcode = a & 0x7F;
        let ins = [a, b, c, d];
        match opcode {
            OPCODE_OP_IMM => op_imm(ins, &mut ints),
            OPCODE_AUIPC => op_auipc(ins, pc, &mut ints),
            OPCODE_OP => op_op(ins, &mut ints),
            OPCODE_LUI => op_lui(ins, &mut ints),
            OPCODE_JAL => op_jal(ins, &mut pc, &mut ints),
            _ => {}
        }
        println!("{:?}", ints);
    }
}

fn op_imm(ins: [u8; 4], ints: &mut IntReg) {
    let (rd, funct3, rs1, imm) = i_type(ins);
    match funct3 {
        FUNCT3_OP_ADD_SUB => addi(rd, rs1, imm, ints),
        FUNCT3_OP_SLL => {
            let imm115 = imm >> 5;
            let shamt = imm & 0x1F;
            match imm115 {
                0 => slli(rd, rs1, shamt, ints),
                _ => {}
            }
        },
        FUNCT3_OP_SLT => slti(rd, rs1, imm, ints),
        FUNCT3_OP_SLTU => sltiu(rd, rs1, imm, ints),
        FUNCT3_OP_XOR => xori(rd, rs1, imm, ints),
        FUNCT3_OP_SRL_SRA => {
            let imm115 = imm >> 5;
            let shamt = imm & 0x1F;
            match imm115 {
                0x00 => srli(rd, rs1, shamt, ints),
                0x20 => srai(rd, rs1, shamt, ints),
                _ => {}
            }
        }
        FUNCT3_OP_OR => ori(rd, rs1, imm, ints),
        FUNCT3_OP_AND => andi(rd, rs1, imm, ints),
        _ => {}
    }
}

#[inline]
fn i_type(ins: [u8; 4]) -> (u8, u8, u8, u32) {
    let rd = ((ins[1] << 1) | (ins[0] >> 7)) & 0x1F;
    let funct3 = (ins[1] >> 4) & 0x07;
    let rs1 = ((ins[2] << 1) | (ins[1] >> 7)) & 0x1F;
    let imm = (ins[2] as u32 >> 4) | ((ins[3] as u32) << 4);
    (rd, funct3, rs1, imm)
}

fn addi(dest: u8, src: u8, imm: u32, ints: &mut IntReg) {
    let imm = if imm & 0x800 != 0 {
        imm | 0xFFFFF000
    } else { imm } as i32;
    let mut value = ints.get(src);
    value = if imm > 0 { 
        value + imm as u32
    } else { 
        value - ((-imm) as u32)
    };
    ints.set(dest, value);
}

fn slti(dest: u8, src: u8, imm: u32, ints: &mut IntReg) {
    let imm = if imm & 0x800 != 0 {
        unsafe { transmute(imm | 0xFFFFF000) }
    } else { imm as i32 };
    let value = ints.get(src);
    let value: i32 = unsafe { transmute(value) };
    let out = if value < imm { 1 } else { 0 };
    ints.set(dest, out);
}

fn sltiu(dest: u8, src: u8, imm: u32, ints: &mut IntReg) {
    let value = ints.get(src);
    let out = if value < imm { 1 } else { 0 };
    ints.set(dest, out);
}

fn andi(dest: u8, src: u8, imm: u32, ints: &mut IntReg) {
    let imm = if imm & 0x800 != 0 {
        imm | 0xFFFFF000
    } else { imm };
    let value = ints.get(src);
    ints.set(dest, value & imm)
}

fn ori(dest: u8, src: u8, imm: u32, ints: &mut IntReg) {
    let imm = if imm & 0x800 != 0 {
        imm | 0xFFFFF000
    } else { imm };
    let value = ints.get(src);
    ints.set(dest, value | imm)
}

fn xori(dest: u8, src: u8, imm: u32, ints: &mut IntReg) {
    let imm = if imm & 0x800 != 0 {
        imm | 0xFFFFF000
    } else { imm };
    let value = ints.get(src);
    ints.set(dest, value ^ imm)
}

fn slli(dest: u8, src: u8, shamt: u32, ints: &mut IntReg) {
    let value = ints.get(src);
    ints.set(dest, value << shamt)
}

fn srli(dest: u8, src: u8, shamt: u32, ints: &mut IntReg) {
    let value = ints.get(src);
    ints.set(dest, value >> shamt)
}

fn srai(dest: u8, src: u8, shamt: u32, ints: &mut IntReg) {
    let value = ints.get(src);
    let value: i32 = unsafe { transmute(value) };
    ints.set(dest, unsafe { transmute(value >> shamt) })
}

fn op_lui(ins: [u8; 4], ints: &mut IntReg) { 
    let (dest, imm) = u_type(ins);
    ints.set(dest, imm)
}

fn op_auipc(ins: [u8; 4], pc: u32, ints: &mut IntReg) { 
    let (dest, imm) = u_type(ins);
    let out = pc + imm;
    ints.set(dest, out)
}

#[inline]
fn u_type(ins: [u8; 4]) -> (u8, u32) {
    let rd = ((ins[1] << 1) | (ins[0] >> 7)) & 0x1F;
    let imm = ((ins[3] as u32) << 24) | 
              ((ins[2] as u32) << 16) | 
              (((ins[1] as u32) >> 4) << 8);
    (rd, imm)
}

fn op_op(ins: [u8; 4], ints: &mut IntReg) { 
    let (dest, funct3, src1, src2, funct7) = r_type(ins);
    match (funct3, funct7) {
        (FUNCT3_OP_ADD_SUB, 0) => add(dest, src1, src2, ints),
        (FUNCT3_OP_SLT, 0) => slt(dest, src1, src2, ints),
        (FUNCT3_OP_SLTU, 0) => sltu(dest, src1, src2, ints),
        (FUNCT3_OP_AND, 0) => and(dest, src1, src2, ints),
        (FUNCT3_OP_OR, 0) => or(dest, src1, src2, ints),
        (FUNCT3_OP_XOR, 0) => xor(dest, src1, src2, ints),
        (FUNCT3_OP_SLL, 0) => sll(dest, src1, src2, ints),
        (FUNCT3_OP_SRL_SRA, 0) => srl(dest, src1, src2, ints),
        (FUNCT3_OP_ADD_SUB, 0x20) => sub(dest, src1, src2, ints),
        (FUNCT3_OP_SRL_SRA, 0x20) => sra(dest, src1, src2, ints),
        _ => {},
    }
}

#[inline]
fn r_type(ins: [u8; 4]) -> (u8, u8, u8, u8, u8) {
    let rd = ((ins[0] >> 7) | (ins[1] << 1)) & 0x1F;
    let funct3 = (ins[1] >> 4) & 0x07;
    let rs1 = ((ins[2] << 1) | (ins[1] >> 7)) & 0x1F;
    let rs2 = (ins[2] >> 4) & 0x1F;
    let funct7 = ins[3] >> 1;
    (rd, funct3, rs1, rs2, funct7)
}

fn add(dest: u8, src1: u8, src2: u8, ints: &mut IntReg) {
    let value1 = ints.get(src1);
    let value2 = ints.get(src2);
    ints.set(dest, value1 + value2);
}

fn slt(dest: u8, src1: u8, src2: u8, ints: &mut IntReg) {
    let value1: i32 = unsafe { transmute(ints.get(src1)) };
    let value2: i32 = unsafe { transmute(ints.get(src2)) };
    ints.set(dest, if value1 < value2 { 1 } else { 0 });
}

fn sltu(dest: u8, src1: u8, src2: u8, ints: &mut IntReg) {
    let value1 = ints.get(src1);
    let value2 = ints.get(src2);
    ints.set(dest, if value1 < value2 { 1 } else { 0 });
}

fn and(dest: u8, src1: u8, src2: u8, ints: &mut IntReg) {
    let value1 = ints.get(src1);
    let value2 = ints.get(src2);
    ints.set(dest, value1 & value2);
}

fn or(dest: u8, src1: u8, src2: u8, ints: &mut IntReg) {
    let value1 = ints.get(src1);
    let value2 = ints.get(src2);
    ints.set(dest, value1 | value2);
}

fn xor(dest: u8, src1: u8, src2: u8, ints: &mut IntReg) {
    let value1 = ints.get(src1);
    let value2 = ints.get(src2);
    ints.set(dest, value1 ^ value2);
}

fn sll(dest: u8, src1: u8, src2: u8, ints: &mut IntReg) {
    let value1 = ints.get(src1);
    let value2 = ints.get(src2);
    ints.set(dest, value1 << (value2 & 0x1F));
}

fn srl(dest: u8, src1: u8, src2: u8, ints: &mut IntReg) {
    let value1 = ints.get(src1);
    let value2 = ints.get(src2);
    ints.set(dest, value1 >> (value2 & 0x1F));
}

fn sra(dest: u8, src1: u8, src2: u8, ints: &mut IntReg) {
    let value1: i32 = unsafe { transmute(ints.get(src1)) };
    let value2: i32 = unsafe { transmute(ints.get(src2)) };
    ints.set(dest, unsafe { transmute(value1 >> (value2 & 0x1F)) });
}

fn sub(dest: u8, src1: u8, src2: u8, ints: &mut IntReg) {
    let value1 = ints.get(src1);
    let value2 = ints.get(src2);
    ints.set(dest, value1 - value2);
}

fn op_jal(ins: [u8; 4], pc: &mut u32, ints: &mut IntReg) {
    let (dest, imm) = j_type(ins);
    if imm & 0x00100000 != 0 {
        *pc -= imm & 0xFFFFF;
    } else {
        *pc += imm;
    }
    ints.set(dest, *pc);
}

#[inline]
fn j_type(ins: [u8; 4]) -> (u8, u32) {
    let rd = ((ins[0] >> 7) | (ins[1] << 1)) & 0x1F;
    let imm = (((ins[1] as u32) >> 4) << 12) |
              ((ins[2] as u32 & 0x0F) << 16) |
              (((ins[2] as u32 & 0x10) >> 4) << 11) |
              ((ins[2] as u32 >> 5) << 1) |
              ((ins[3] as u32 & 0x7F) << 4) | 
              (((ins[3] as u32 & 0x80) >> 7) << 20);
    (rd, imm)
}

// #[inline]
// fn s_type(ins: [u8; 4]) -> (u32, u8, u8, u8) {
//     let funct3 = (ins[1] >> 4) & 0x07;
//     let rs1 = ((ins[2] << 1) | (ins[1] >> 7)) & 0x1F;
//     let rs2 = (ins[2] >> 4) & 0x1F;
//     let imm = (ins[0] as u32 >> 7) |
//               (((ins[1] as u32) & 0x0F) << 1) |
//               (((ins[3] as u32) >> 1) << 5);
//     (imm, funct3, rs1, rs2)
// }
