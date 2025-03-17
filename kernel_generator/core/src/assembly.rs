use std::fmt::Write;

#[derive(Clone, Copy)]
pub enum PrefetchType {
    T0,
    T1,
    T2,
    NTA,
}

pub struct Assembly {
    arr: Vec<Instruction>,
    var_asms: Vec<(&'static str, &'static str)>,
    zmm_used: [bool; 32],
    k_used: [bool; 4],
}

const fn is_comment(asm: &str) -> bool {
    let arr = asm.as_bytes();
    asm.len() > 2 && arr[0] == b'/' && arr[1] == b'/'
}

enum Instruction {
    Comment(&'static str),
    Nop,
    Label(&'static str),
    JumpNotZero(&'static str), // jnz loop0

    MaskOn(u8),
    MaskSet(u8, &'static str), // kmovw reg_src, k_dst
    MaskNot(u8, u8),           // knotw k_src, k_dst

    AddImmediate(&'static str, i16),
    SubImmediate(&'static str, i16), // sub $0x1, %[J]
    SetImmediate(&'static str, i16),
    ShiftRight(&'static str, u8), // sar $0x3,%edx or sar %edx
    MovReg(&'static str, &'static str),

    MovF64x8(u8, u8),
    MovF64x2(u8, u8),
    MovI32x8(u8, u8),

    LoadF64x8(u8, &'static str, i16),
    LoadF64x1(u8, &'static str, i16),
    LoadI32x8(u8, &'static str, i16),
    StoreF64x8(&'static str, i16, u8),
    StoreF64x1(&'static str, i16, u8),
    GatherF64x8(u8, &'static str, u8, u8),
    Prefetch(PrefetchType, &'static str, i16),

    AddF64x8(u8, u8, u8),
    AddF64x4(u8, u8, u8),
    AddF64x2(u8, u8, u8),
    AddF64x1(u8, u8, u8),
    LoadAddF64x8(u8, u8, &'static str, i16),
    LoadAddF64x1(u8, u8, &'static str, i16),
    MulF64x8(u8, u8, u8),
    MulF64x1(u8, u8, u8),
    LoadMulF64x8(u8, u8, &'static str, i16),
    LoadMulF64x1(u8, u8, &'static str, i16),
    MulAddF64x8(u8, u8, u8),
    MulAddF64x1(u8, u8, u8),
    LoadMulAddF64x8(u8, u8, &'static str, i16),
    LoadMulAddF64x1(u8, u8, &'static str, i16),
    NMulSubF64x8(u8, u8, u8),
    LoadNMulSubF64x8(u8, u8, &'static str, i16),
    DivF64x8(u8, u8, u8),
    DivF64x1(u8, u8, u8),
    LoadDivF64x8(u8, u8, &'static str, i16),
    LoadDivF64x1(u8, u8, &'static str, i16),

    LUMix4F64x8(u8, u8, u8), // vinsertf64x4 $0x0, ymm_src2, zmm_src1, zmm_dst = lower src1 & upper src2
    ULMix4F64x8(u8, u8, u8), // valignq $0x4, zmm_src2, zmm_src1, zmm_dst = upper src1 & lower src2
    Mix2F64x8Mask(u8, u8, u8), // vpermpd $0x4e, zmm_src, zmm_dst%{%%k%}
    LUMix1F64x8(u8, u8, u8), // vshufpd $0xaa, zmm_src2, zmm_src1, zmm_dst \t\n"
    ULMix1F64x8(u8, u8, u8), // vshufpd $0x55, zmm_src2, zmm_src1, zmm_dst \t\n"
    ExtractU4F64x8(u8, u8),  // vextractf128 $0x1, zmm_src, ymm_dst
    ExtractU2F64x4(u8, u8),  // vextractf128 $0x1, ymm_src, xmm_dst
    Fold1AddF64x2(u8, u8),   // vhaddpd xmm_src, xmm_src, xmm_dst
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Instruction::Comment(comment) => match comment {
                "" => write!(f, ""),
                _ => write!(f, "// {}", comment),
            },
            Instruction::Nop => write!(f, "nop"),
            Instruction::Label(name) => write!(f, "{}:", name),
            Instruction::JumpNotZero(label) => write!(f, "jnz {}", label),

            Instruction::MaskOn(k) => write!(f, "kxnorw %%k0, %%k0, %%k{}", k),
            Instruction::MaskSet(k, reg_name) => write!(f, "kmovw %[{}], %%k{}", reg_name, k),
            Instruction::MaskNot(k_dst, k_src) => write!(f, "knotw %%k{}, %%k{}", k_src, k_dst),

            Instruction::AddImmediate(reg_name, imm) => match imm > 0 {
                true => write!(f, "add $0x{:x}, %[{}]", imm, reg_name),
                false => write!(f, "add $-0x{:x}, %[{}]", -imm, reg_name),
            },
            Instruction::SubImmediate(reg_name, imm) => match imm > 0 {
                true => write!(f, "sub $0x{:x}, %[{}]", imm, reg_name),
                false => write!(f, "sub $-0x{:x}, %[{}]", -imm, reg_name),
            },
            Instruction::SetImmediate(reg_name, imm) => match imm > 0 {
                true => write!(f, "movl $0x{:x}, %[{}]", imm, reg_name),
                false => write!(f, "movl $-0x{:x}, %[{}]", -imm, reg_name),
            },
            Instruction::ShiftRight(reg_name, imm) => match imm {
                1 => write!(f, "sar %[{}]", reg_name),
                _ => write!(f, "sar $0x{:x}, %[{}]", imm, reg_name),
            },
            Instruction::MovReg(reg_dist, reg_src) => {
                write!(f, "mov %[{}], %[{}]", reg_src, reg_dist)
            }

            Instruction::MovF64x8(zmm_dst, zmm_src) => {
                write!(f, "vmovupd %%zmm{}, %%zmm{}", zmm_src, zmm_dst)
            }
            Instruction::MovF64x2(xmm_dst, xmm_src) => {
                write!(f, "vmovupd %%xmm{}, %%xmm{}", xmm_src, xmm_dst)
            }
            Instruction::MovI32x8(ymm_dst, ymm_src) => {
                write!(f, "vmovupd %%ymm{}, %%ymm{}", ymm_src, ymm_dst)
            }

            Instruction::LoadF64x8(zmm, reg_base, imm_offset) => match imm_offset {
                0 => write!(f, "vmovupd (%[{}]), %%zmm{}", reg_base, zmm),
                imm if imm > 0 => write!(f, "vmovupd 0x{:x}(%[{}]), %%zmm{}", imm, reg_base, zmm),
                imm => write!(f, "vmovupd -0x{:x}(%[{}]), %%zmm{}", -imm, reg_base, zmm),
            },
            Instruction::LoadF64x1(xmm, reg_base, imm_offset) => match imm_offset {
                0 => write!(f, "vmovsd (%[{}]), %%xmm{}", reg_base, xmm),
                imm if imm > 0 => write!(f, "vmovsd 0x{:x}(%[{}]), %%xmm{}", imm, reg_base, xmm),
                imm => write!(f, "vmovsd -0x{:x}(%[{}]), %%xmm{}", -imm, reg_base, xmm),
            },
            Instruction::LoadI32x8(ymm, reg_base, imm_offset) => match imm_offset {
                0 => write!(f, "vmovdqa (%[{}]), %%ymm{}", reg_base, ymm),
                imm if imm > 0 => write!(f, "vmovdqa 0x{:x}(%[{}]), %%ymm{}", imm, reg_base, ymm),
                imm => write!(f, "vmovdqa -0x{:x}(%[{}]), %%ymm{}", -imm, reg_base, ymm),
            },
            Instruction::StoreF64x8(reg_base, imm_offset, zmm) => match imm_offset {
                0 => write!(f, "vmovupd %%zmm{}, (%[{}])", zmm, reg_base),
                imm if imm > 0 => write!(f, "vmovupd %%zmm{}, 0x{:x}(%[{}])", zmm, imm, reg_base),
                imm => write!(f, "vmovupd %%zmm{}, -0x{:x}(%[{}])", zmm, -imm, reg_base),
            },
            Instruction::StoreF64x1(reg_base, imm_offset, xmm) => match imm_offset {
                0 => write!(f, "vmovsd %%xmm{}, (%[{}])", xmm, reg_base),
                imm if imm > 0 => write!(f, "vmovsd %%xmm{}, 0x{:x}(%[{}])", xmm, imm, reg_base),
                imm => write!(f, "vmovsd %%xmm{}, -0x{:x}(%[{}])", xmm, -imm, reg_base),
            },
            Instruction::GatherF64x8(zmm, reg_base, ymm_idx, k) => write!(
                f,
                "vgatherdpd (%[{}],%%ymm{},8), %%zmm{}%{{%%k{}%}}",
                reg_base, ymm_idx, zmm, k
            ),
            Instruction::Prefetch(prefetch_type, reg_base, imm_offset) => {
                let inst = match prefetch_type {
                    PrefetchType::NTA => "prefetchnta",
                    PrefetchType::T0 => "prefetcht0",
                    PrefetchType::T1 => "prefetcht1",
                    PrefetchType::T2 => "prefetcht2",
                };
                match imm_offset > 0 {
                    true => write!(f, "{} 0x{:x}(%[{}])", inst, imm_offset, reg_base),
                    false => write!(f, "{} -0x{:x}(%[{}])", inst, -imm_offset, reg_base),
                }
            }

            Instruction::AddF64x8(zmm_dst, zmm_src0, zmm_src1) => {
                write!(
                    f,
                    "vaddpd %%zmm{}, %%zmm{}, %%zmm{}",
                    zmm_src1, zmm_src0, zmm_dst
                )
            }
            Instruction::AddF64x4(ymm_dst, ymm_src0, ymm_src1) => {
                write!(
                    f,
                    "vaddpd %%ymm{}, %%ymm{}, %%ymm{}",
                    ymm_src1, ymm_src0, ymm_dst
                )
            }
            Instruction::AddF64x2(xmm_dst, xmm_src0, xmm_src1) => {
                write!(
                    f,
                    "vaddpd %%xmm{}, %%xmm{}, %%xmm{}",
                    xmm_src1, xmm_src0, xmm_dst
                )
            }
            Instruction::AddF64x1(xmm_dst, xmm_src0, xmm_src1) => {
                write!(
                    f,
                    "vaddsd %%xmm{}, %%xmm{}, %%xmm{}",
                    xmm_src1, xmm_src0, xmm_dst
                )
            }
            Instruction::LoadAddF64x8(zmm_dst, zmm_src0, reg_base1, imm_offset1) => {
                match imm_offset1 {
                    0 => write!(
                        f,
                        "vaddpd (%[{}]), %%zmm{}, %%zmm{}",
                        reg_base1, zmm_src0, zmm_dst
                    ),
                    imm if imm > 0 => write!(
                        f,
                        "vaddpd 0x{:x}(%[{}]), %%zmm{}, %%zmm{}",
                        imm, reg_base1, zmm_src0, zmm_dst
                    ),
                    imm => write!(
                        f,
                        "vaddpd -0x{:x}(%[{}]), %%zmm{}, %%zmm{}",
                        -imm, reg_base1, zmm_src0, zmm_dst
                    ),
                }
            }
            Instruction::LoadAddF64x1(xmm_dst, xmm_src0, reg_base1, imm_offset1) => {
                match imm_offset1 {
                    0 => write!(
                        f,
                        "vaddsd (%[{}]), %%xmm{}, %%xmm{}",
                        reg_base1, xmm_src0, xmm_dst
                    ),
                    imm if imm > 0 => write!(
                        f,
                        "vaddsd 0x{:x}(%[{}]), %%xmm{}, %%xmm{}",
                        imm, reg_base1, xmm_src0, xmm_dst
                    ),
                    imm => write!(
                        f,
                        "vaddsd -0x{:x}(%[{}]), %%xmm{}, %%xmm{}",
                        -imm, reg_base1, xmm_src0, xmm_dst
                    ),
                }
            }
            Instruction::MulF64x8(zmm_dst, zmm_src0, zmm_src1) => {
                write!(
                    f,
                    "vmulpd %%zmm{}, %%zmm{}, %%zmm{}",
                    zmm_src1, zmm_src0, zmm_dst
                )
            }
            Instruction::MulF64x1(xmm_dst, xmm_src0, xmm_src1) => {
                write!(
                    f,
                    "vmulsd %%xmm{}, %%xmm{}, %%xmm{}",
                    xmm_src1, xmm_src0, xmm_dst
                )
            }
            Instruction::LoadMulF64x8(zmm_dst, zmm_src0, reg_base1, imm_offset1) => {
                match imm_offset1 {
                    0 => write!(
                        f,
                        "vmulpd (%[{}]), %%zmm{}, %%zmm{}",
                        reg_base1, zmm_src0, zmm_dst
                    ),
                    imm if imm > 0 => write!(
                        f,
                        "vmulpd 0x{:x}(%[{}]), %%zmm{}, %%zmm{}",
                        imm, reg_base1, zmm_src0, zmm_dst
                    ),
                    imm => write!(
                        f,
                        "vmulpd -0x{:x}(%[{}]), %%zmm{}, %%zmm{}",
                        -imm, reg_base1, zmm_src0, zmm_dst
                    ),
                }
            }
            Instruction::LoadMulF64x1(xmm_dst, xmm_src0, reg_base1, imm_offset1) => {
                match imm_offset1 {
                    0 => write!(
                        f,
                        "vmulsd (%[{}]), %%xmm{}, %%xmm{}",
                        reg_base1, xmm_src0, xmm_dst
                    ),
                    imm if imm > 0 => write!(
                        f,
                        "vmulsd 0x{:x}(%[{}]), %%xmm{}, %%xmm{}",
                        imm, reg_base1, xmm_src0, xmm_dst
                    ),
                    imm => write!(
                        f,
                        "vmulsd -0x{:x}(%[{}]), %%xmm{}, %%xmm{}",
                        -imm, reg_base1, xmm_src0, xmm_dst
                    ),
                }
            }
            Instruction::MulAddF64x8(zmm_dst, zmm_src0, zmm_src1) => {
                write!(
                    f,
                    "vfmadd231pd %%zmm{}, %%zmm{}, %%zmm{}",
                    zmm_src1, zmm_src0, zmm_dst
                )
            }
            Instruction::MulAddF64x1(xmm_dst, xmm_src0, xmm_src1) => {
                write!(
                    f,
                    "vfmadd231sd %%xmm{}, %%xmm{}, %%xmm{}",
                    xmm_src1, xmm_src0, xmm_dst
                )
            }
            Instruction::LoadMulAddF64x8(zmm_dst, zmm_src0, reg_base1, imm_offset1) => {
                match imm_offset1 {
                    0 => write!(
                        f,
                        "vfmadd231pd (%[{}]), %%zmm{}, %%zmm{}",
                        reg_base1, zmm_src0, zmm_dst
                    ),
                    imm if imm > 0 => write!(
                        f,
                        "vfmadd231pd 0x{:x}(%[{}]), %%zmm{}, %%zmm{}",
                        imm, reg_base1, zmm_src0, zmm_dst
                    ),
                    imm => write!(
                        f,
                        "vfmadd231pd -0x{:x}(%[{}]), %%zmm{}, %%zmm{}",
                        -imm, reg_base1, zmm_src0, zmm_dst
                    ),
                }
            }
            Instruction::LoadMulAddF64x1(xmm_dst, xmm_src0, reg_base1, imm_offset1) => {
                match imm_offset1 {
                    0 => write!(
                        f,
                        "vfmadd231sd (%[{}]), %%xmm{}, %%xmm{}",
                        reg_base1, xmm_src0, xmm_dst
                    ),
                    imm if imm > 0 => write!(
                        f,
                        "vfmadd231sd 0x{:x}(%[{}]), %%xmm{}, %%xmm{}",
                        imm, reg_base1, xmm_src0, xmm_dst
                    ),
                    imm => write!(
                        f,
                        "vfmadd231sd -0x{:x}(%[{}]), %%xmm{}, %%xmm{}",
                        -imm, reg_base1, xmm_src0, xmm_dst
                    ),
                }
            }
            Instruction::NMulSubF64x8(zmm_dst, zmm_src0, zmm_src1) => {
                write!(
                    f,
                    "vfnmsub231pd %%zmm{}, %%zmm{}, %%zmm{}",
                    zmm_src1, zmm_src0, zmm_dst
                )
            }
            Instruction::LoadNMulSubF64x8(zmm_dst, zmm_src0, reg_base1, imm_offset1) => {
                match imm_offset1 {
                    0 => write!(
                        f,
                        "vfnmsub231pd (%[{}]), %%zmm{}, %%zmm{}",
                        reg_base1, zmm_src0, zmm_dst
                    ),
                    imm if imm > 0 => write!(
                        f,
                        "vfnmsub231pd 0x{:x}(%[{}]), %%zmm{}, %%zmm{}",
                        imm, reg_base1, zmm_src0, zmm_dst
                    ),
                    imm => write!(
                        f,
                        "vfnmsub231pd -0x{:x}(%[{}]), %%zmm{}, %%zmm{}",
                        -imm, reg_base1, zmm_src0, zmm_dst
                    ),
                }
            }
            Instruction::DivF64x8(zmm_dst, zmm_src0, zmm_src1) => {
                write!(
                    f,
                    "vdivpd %%zmm{}, %%zmm{}, %%zmm{}",
                    zmm_src1, zmm_src0, zmm_dst
                )
            }
            Instruction::DivF64x1(xmm_dst, xmm_src0, xmm_src1) => {
                write!(
                    f,
                    "vdivsd %%xmm{}, %%xmm{}, %%xmm{}",
                    xmm_src1, xmm_src0, xmm_dst
                )
            }
            Instruction::LoadDivF64x8(zmm_dst, zmm_src0, reg_base1, imm_offset1) => {
                match imm_offset1 {
                    0 => write!(
                        f,
                        "vdivpd (%[{}]), %%zmm{}, %%zmm{}",
                        reg_base1, zmm_src0, zmm_dst
                    ),
                    imm if imm > 0 => write!(
                        f,
                        "vdivpd 0x{:x}(%[{}]), %%zmm{}, %%zmm{}",
                        imm, reg_base1, zmm_src0, zmm_dst
                    ),
                    imm => write!(
                        f,
                        "vdivpd -0x{:x}(%[{}]), %%zmm{}, %%zmm{}",
                        -imm, reg_base1, zmm_src0, zmm_dst
                    ),
                }
            }
            Instruction::LoadDivF64x1(xmm_dst, xmm_src0, reg_base1, imm_offset1) => {
                match imm_offset1 {
                    0 => write!(
                        f,
                        "vdivsd (%[{}]), %%xmm{}, %%xmm{}",
                        reg_base1, xmm_src0, xmm_dst
                    ),
                    imm if imm > 0 => write!(
                        f,
                        "vdivsd 0x{:x}(%[{}]), %%xmm{}, %%xmm{}",
                        imm, reg_base1, xmm_src0, xmm_dst
                    ),
                    imm => write!(
                        f,
                        "vdivsd -0x{:x}(%[{}]), %%xmm{}, %%xmm{}",
                        -imm, reg_base1, xmm_src0, xmm_dst
                    ),
                }
            }

            Instruction::LUMix4F64x8(zmm_dst, ymm_src0, zmm_src1) => write!(
                f,
                "vinsertf64x4 $0x0, %%ymm{}, %%zmm{}, %%zmm{}",
                ymm_src0, zmm_src1, zmm_dst
            ),
            Instruction::ULMix4F64x8(zmm_dst, zmm_src0, zmm_src1) => write!(
                f,
                "valignq $0x4, %%zmm{}, %%zmm{}, %%zmm{}",
                zmm_src0, zmm_src1, zmm_dst
            ),
            Instruction::Mix2F64x8Mask(zmm_dst, zmm_src, k) => write!(
                f,
                "vpermpd $0x4e, %%zmm{}, %%zmm{}%{{%%k{}%}}",
                zmm_src, zmm_dst, k
            ),
            Instruction::LUMix1F64x8(zmm_dst, zmm_src0, zmm_src1) => write!(
                f,
                "vshufpd $0xaa, %%zmm{}, %%zmm{}, %%zmm{}",
                zmm_src1, zmm_src0, zmm_dst
            ),
            Instruction::ULMix1F64x8(zmm_dst, zmm_src0, zmm_src1) => write!(
                f,
                "vshufpd $0x55, %%zmm{}, %%zmm{}, %%zmm{}",
                zmm_src1, zmm_src0, zmm_dst
            ),
            Instruction::ExtractU4F64x8(ymm_dst, zmm_src) => {
                write!(f, "vextractf64x4 $0x1, %%zmm{}, %%ymm{}", zmm_src, ymm_dst)
            }
            Instruction::ExtractU2F64x4(xmm_dst, ymm_src) => {
                write!(f, "vextractf128 $0x1, %%ymm{}, %%xmm{}", ymm_src, xmm_dst)
            }
            Instruction::Fold1AddF64x2(xmm_dst, xmm_src) => {
                write!(
                    f,
                    "vhaddpd %%xmm{}, %%xmm{}, %%xmm{}",
                    xmm_src, xmm_src, xmm_dst
                )
            }
        }
    }
}

impl Assembly {
    pub fn new() -> Self {
        Assembly {
            arr: Vec::new(),
            var_asms: Vec::new(),
            zmm_used: [false; 32],
            k_used: [false; 4],
        }
    }

    pub fn var_asm(mut self, var: &'static str, asm: &'static str) -> Self {
        self.var_asms.push((var, asm));
        self
    }

    pub fn print(
        &self,
        tab: usize,
        variable_names: &[&'static str],
        asm_names: &[&'static str],
    ) -> String {
        let tab = "    ".repeat(tab);
        let mut output = String::new();

        write!(output, "{}asm volatile(\n", tab).unwrap();

        for inst in self.arr.iter() {
            let asm = inst.to_string();
            match asm.as_str() {
                "" => write!(output, "\n").unwrap(),
                asm if is_comment(asm) => write!(output, "{}{:48}\n", tab, asm).unwrap(),
                _ => write!(output, "{}\" {:48} \\t\\n\"\n", tab, asm).unwrap(),
            }
        }

        let output_operands = variable_names
            .iter()
            .zip(asm_names.iter())
            .map(|(var, asm)| format!("[{}]\"+r\"({})", *asm, *var))
            .reduce(|mut acc, x| {
                acc.reserve(x.len() + 2);
                acc.push_str(", ");
                acc.push_str(&x);
                acc
            })
            .unwrap_or(String::new());
        write!(output, "{}: {}\n", tab, output_operands).unwrap();

        write!(output, "{}:\n", tab).unwrap();

        let iter_zmm = (0..32)
            .filter(|i| self.zmm_used[*i])
            .map(|i| format!("\"zmm{}\"", i));
        let iter_k = (0..4)
            .filter(|i| self.k_used[*i])
            .map(|i| format!("\"k{}\"", i + 1));
        let clobbers = iter_zmm
            .chain(iter_k)
            .reduce(|mut acc, x| {
                acc.reserve(x.len() + 2);
                acc.push_str(", ");
                acc.push_str(&x);
                acc
            })
            .unwrap_or(String::new());
        write!(output, "{}: {}\n", tab, clobbers).unwrap();
        write!(output, "{});\n", tab).unwrap();

        output
    }

    pub fn append(mut self, other: Self) -> Self {
        let mut other = other;
        self.arr.append(&mut other.arr);
        self.var_asms.append(&mut other.var_asms);

        for i in 0..32 {
            self.zmm_used[i] = self.zmm_used[i] | other.zmm_used[i];
        }
        for i in 0..4 {
            self.k_used[i] = self.k_used[i] | other.k_used[i];
        }

        self
    }

    pub fn empty_line(mut self) -> Self {
        self.arr.push(Instruction::Comment(""));
        self
    }

    pub fn comment(mut self, comment: &'static str) -> Self {
        self.arr.push(Instruction::Comment(comment));
        self
    }

    pub fn nop(mut self, size: u8) -> Self {
        for _ in 0..size {
            self.arr.push(Instruction::Nop);
        }
        self
    }

    pub fn label(mut self, name: &'static str) -> Self {
        self.arr.push(Instruction::Label(name));
        self
    }

    pub fn jump_nz(mut self, name: &'static str) -> Self {
        self.arr.push(Instruction::JumpNotZero(name));
        self
    }

    pub fn mask_on(mut self, k: u8) -> Self {
        self.arr.push(Instruction::MaskOn(k));
        self.k_used[(k - 1) as usize] = true;
        self
    }

    pub fn add_immediate(mut self, reg_name: &'static str, imm: i16) -> Self {
        self.arr.push(Instruction::AddImmediate(reg_name, imm));
        self
    }

    pub fn sub_immediate(mut self, reg_name: &'static str, imm: i16) -> Self {
        self.arr.push(Instruction::SubImmediate(reg_name, imm));
        self
    }

    pub fn set_immediate(mut self, reg_name: &'static str, imm: i16) -> Self {
        self.arr.push(Instruction::SetImmediate(reg_name, imm));
        self
    }

    pub fn shift_right(mut self, reg_name: &'static str, imm: u8) -> Self {
        self.arr.push(Instruction::ShiftRight(reg_name, imm));
        self
    }

    pub fn move_reg(mut self, reg_dst: &'static str, reg_src: &'static str) -> Assembly {
        self.arr.push(Instruction::MovReg(reg_dst, reg_src));
        self
    }

    pub fn move_f64x8(mut self, zmm_dst: u8, zmm_src: u8) -> Assembly {
        self.arr.push(Instruction::MovF64x8(zmm_dst, zmm_src));
        self.zmm_used[zmm_dst as usize] = true;
        self
    }

    pub fn move_f64x2(mut self, xmm_dst: u8, xmm_src: u8) -> Assembly {
        self.arr.push(Instruction::MovF64x2(xmm_dst, xmm_src));
        self.zmm_used[xmm_dst as usize] = true;
        self
    }

    pub fn move_i32x8(mut self, ymm_dst: u8, ymm_src: u8) -> Assembly {
        self.arr.push(Instruction::MovI32x8(ymm_dst, ymm_src));
        self.zmm_used[ymm_dst as usize] = true;
        self
    }

    pub fn load_f64x8(mut self, zmm: u8, reg_name: &'static str, base: i16) -> Self {
        self.arr.push(Instruction::LoadF64x8(zmm, reg_name, base));
        self.zmm_used[zmm as usize] = true;
        self
    }

    pub fn load_f64x1(mut self, xmm: u8, reg_name: &'static str, base: i16) -> Self {
        self.arr.push(Instruction::LoadF64x1(xmm, reg_name, base));
        self.zmm_used[xmm as usize] = true;
        self
    }

    pub fn load_i32x8(mut self, ymm: u8, reg_name: &'static str, base: i16) -> Self {
        assert!(ymm < 16, "VEX instruction can only use ymm less than 16");
        self.arr.push(Instruction::LoadI32x8(ymm, reg_name, base));
        self.zmm_used[ymm as usize] = true;
        self
    }

    pub fn store_f64x8(mut self, reg_name: &'static str, base: i16, zmm: u8) -> Self {
        self.arr.push(Instruction::StoreF64x8(reg_name, base, zmm));
        self
    }

    pub fn store_f64x1(mut self, reg_name: &'static str, base: i16, xmm: u8) -> Self {
        self.arr.push(Instruction::StoreF64x1(reg_name, base, xmm));
        self
    }

    pub fn gather_f64x8(mut self, zmm: u8, reg_name: &'static str, ymm_idx: u8, k: u8) -> Self {
        assert!(
            zmm != ymm_idx,
            "Operands `dst` and `src_idx` of VGATHERDPD must be different."
        );
        self.arr
            .push(Instruction::GatherF64x8(zmm, reg_name, ymm_idx, k));
        self.zmm_used[zmm as usize] = true;
        self
    }

    pub fn prefetch(
        mut self,
        prefetch_type: PrefetchType,
        reg_name: &'static str,
        base: i16,
    ) -> Self {
        self.arr
            .push(Instruction::Prefetch(prefetch_type, reg_name, base));
        self
    }

    pub fn add_f64x8(mut self, zmm_dst: u8, zmm_src0: u8, zmm_src1: u8) -> Assembly {
        self.arr
            .push(Instruction::AddF64x8(zmm_dst, zmm_src0, zmm_src1));
        self.zmm_used[zmm_dst as usize] = true;
        self
    }

    pub fn add_f64x1(mut self, xmm_dst: u8, xmm_src0: u8, xmm_src1: u8) -> Assembly {
        self.arr
            .push(Instruction::AddF64x1(xmm_dst, xmm_src0, xmm_src1));
        self.zmm_used[xmm_dst as usize] = true;
        self
    }

    pub fn loadadd_f64x8(
        mut self,
        zmm_dst: u8,
        zmm_src0: u8,
        reg_name_src1: &'static str,
        base_src1: i16,
    ) -> Assembly {
        self.arr.push(Instruction::LoadAddF64x8(
            zmm_dst,
            zmm_src0,
            reg_name_src1,
            base_src1,
        ));
        self.zmm_used[zmm_dst as usize] = true;
        self
    }

    pub fn loadadd_f64x1(
        mut self,
        xmm_dst: u8,
        xmm_src0: u8,
        reg_name_src1: &'static str,
        base_src1: i16,
    ) -> Assembly {
        self.arr.push(Instruction::LoadAddF64x1(
            xmm_dst,
            xmm_src0,
            reg_name_src1,
            base_src1,
        ));
        self.zmm_used[xmm_dst as usize] = true;
        self
    }

    pub fn mul_f64x8(mut self, zmm_dst: u8, zmm_src0: u8, zmm_src1: u8) -> Assembly {
        self.arr
            .push(Instruction::MulF64x8(zmm_dst, zmm_src0, zmm_src1));
        self.zmm_used[zmm_dst as usize] = true;
        self
    }

    pub fn mul_f64x1(mut self, xmm_dst: u8, xmm_src0: u8, xmm_src1: u8) -> Assembly {
        self.arr
            .push(Instruction::MulF64x1(xmm_dst, xmm_src0, xmm_src1));
        self.zmm_used[xmm_dst as usize] = true;
        self
    }

    pub fn loadmul_f64x8(
        mut self,
        zmm_dst: u8,
        zmm_src0: u8,
        reg_name_src1: &'static str,
        base_src1: i16,
    ) -> Assembly {
        self.arr.push(Instruction::LoadMulF64x8(
            zmm_dst,
            zmm_src0,
            reg_name_src1,
            base_src1,
        ));
        self.zmm_used[zmm_dst as usize] = true;
        self
    }

    pub fn loadmul_f64x1(
        mut self,
        xmm_dst: u8,
        xmm_src0: u8,
        reg_name_src1: &'static str,
        base_src1: i16,
    ) -> Assembly {
        self.arr.push(Instruction::LoadMulF64x1(
            xmm_dst,
            xmm_src0,
            reg_name_src1,
            base_src1,
        ));
        self.zmm_used[xmm_dst as usize] = true;
        self
    }

    pub fn muladd_f64x8(mut self, zmm_dst: u8, zmm_src0: u8, zmm_src1: u8) -> Assembly {
        self.arr
            .push(Instruction::MulAddF64x8(zmm_dst, zmm_src0, zmm_src1));
        self.zmm_used[zmm_dst as usize] = true;
        self
    }

    pub fn muladd_f64x1(mut self, xmm_dst: u8, xmm_src0: u8, xmm_src1: u8) -> Assembly {
        self.arr
            .push(Instruction::MulAddF64x1(xmm_dst, xmm_src0, xmm_src1));
        self.zmm_used[xmm_dst as usize] = true;
        self
    }

    pub fn loadmuladd_f64x8(
        mut self,
        zmm_dst: u8,
        zmm_src0: u8,
        reg_src1: &'static str,
        base_src1: i16,
    ) -> Assembly {
        self.arr.push(Instruction::LoadMulAddF64x8(
            zmm_dst, zmm_src0, reg_src1, base_src1,
        ));
        self.zmm_used[zmm_dst as usize] = true;
        self
    }

    pub fn loadmuladd_f64x1(
        mut self,
        xmm_dst: u8,
        xmm_src0: u8,
        reg_src1: &'static str,
        base_src1: i16,
    ) -> Assembly {
        self.arr.push(Instruction::LoadMulAddF64x1(
            xmm_dst, xmm_src0, reg_src1, base_src1,
        ));
        self.zmm_used[xmm_dst as usize] = true;
        self
    }

    pub fn nmulsub_f64x8(mut self, zmm_dst: u8, zmm_src0: u8, zmm_src1: u8) -> Assembly {
        self.arr
            .push(Instruction::NMulSubF64x8(zmm_dst, zmm_src0, zmm_src1));
        self.zmm_used[zmm_dst as usize] = true;
        self
    }

    pub fn loadnmulsub_f64x8(
        mut self,
        zmm_dst: u8,
        zmm_src0: u8,
        reg_src1: &'static str,
        base_src1: i16,
    ) -> Assembly {
        self.arr.push(Instruction::LoadNMulSubF64x8(
            zmm_dst, zmm_src0, reg_src1, base_src1,
        ));
        self.zmm_used[zmm_dst as usize] = true;
        self
    }

    pub fn div_f64x8(mut self, zmm_dst: u8, zmm_src0: u8, zmm_src1: u8) -> Assembly {
        self.arr
            .push(Instruction::DivF64x8(zmm_dst, zmm_src0, zmm_src1));
        self.zmm_used[zmm_dst as usize] = true;
        self
    }

    pub fn div_f64x1(mut self, xmm_dst: u8, xmm_src0: u8, xmm_src1: u8) -> Assembly {
        self.arr
            .push(Instruction::DivF64x1(xmm_dst, xmm_src0, xmm_src1));
        self.zmm_used[xmm_dst as usize] = true;
        self
    }

    pub fn loaddiv_f64x8(
        mut self,
        zmm_dst: u8,
        zmm_src0: u8,
        reg_name_src1: &'static str,
        base_src1: i16,
    ) -> Assembly {
        self.arr.push(Instruction::LoadDivF64x8(
            zmm_dst,
            zmm_src0,
            reg_name_src1,
            base_src1,
        ));
        self.zmm_used[zmm_dst as usize] = true;
        self
    }

    pub fn loaddiv_f64x1(
        mut self,
        xmm_dst: u8,
        xmm_src0: u8,
        reg_name_src1: &'static str,
        base_src1: i16,
    ) -> Assembly {
        self.arr.push(Instruction::LoadDivF64x1(
            xmm_dst,
            xmm_src0,
            reg_name_src1,
            base_src1,
        ));
        self.zmm_used[xmm_dst as usize] = true;
        self
    }

    pub fn mix4add_f64x8(mut self, zmm_dst: u8, zmm_src0: u8, zmm_src1: u8) -> Assembly {
        self.arr
            .push(Instruction::LUMix4F64x8(zmm_dst, zmm_src0, zmm_src1));
        self.arr
            .push(Instruction::ULMix4F64x8(zmm_src1, zmm_src0, zmm_src1));
        self.arr
            .push(Instruction::AddF64x8(zmm_dst, zmm_dst, zmm_src1));
        self.zmm_used[zmm_dst as usize] = true;
        self.zmm_used[zmm_src1 as usize] = true;
        self
    }

    pub fn init_mix2mask(mut self, reg_name: &'static str, mask0: u8, mask1: u8) -> Assembly {
        self.arr.push(Instruction::SetImmediate(reg_name, 0x33));
        self.arr.push(Instruction::MaskSet(mask0, reg_name));
        self.arr.push(Instruction::MaskNot(mask1, mask0));
        self.k_used[(mask0 - 1) as usize] = true;
        self.k_used[(mask1 - 1) as usize] = true;
        self
    }

    pub fn mix2add_f64x8(
        mut self,
        zmm_dst: u8,
        zmm_src0: u8,
        zmm_src1: u8,
        mask0: u8,
        mask1: u8,
    ) -> Assembly {
        self.arr.push(Instruction::MovF64x8(zmm_dst, zmm_src1));
        self.arr
            .push(Instruction::Mix2F64x8Mask(zmm_src1, zmm_src0, mask0));
        self.arr
            .push(Instruction::Mix2F64x8Mask(zmm_src0, zmm_dst, mask1));
        self.arr
            .push(Instruction::AddF64x8(zmm_dst, zmm_src0, zmm_src1));
        self.zmm_used[zmm_dst as usize] = true;
        self.zmm_used[zmm_src0 as usize] = true;
        self.zmm_used[zmm_src1 as usize] = true;
        self
    }

    pub fn mix1add_f64x8(mut self, zmm_dst: u8, zmm_src0: u8, zmm_src1: u8) -> Assembly {
        self.arr
            .push(Instruction::LUMix1F64x8(zmm_dst, zmm_src0, zmm_src1));
        self.arr
            .push(Instruction::ULMix1F64x8(zmm_src0, zmm_src0, zmm_src1));
        self.arr
            .push(Instruction::AddF64x8(zmm_dst, zmm_dst, zmm_src0));
        self.zmm_used[zmm_dst as usize] = true;
        self.zmm_used[zmm_src0 as usize] = true;
        self
    }

    pub fn fold4add_f64x8(mut self, ymm_dst: u8, zmm_src: u8) -> Assembly {
        assert!(ymm_dst != zmm_src);

        self.arr.push(Instruction::ExtractU4F64x8(ymm_dst, zmm_src));
        self.arr
            .push(Instruction::AddF64x4(ymm_dst, ymm_dst, zmm_src));
        self.zmm_used[ymm_dst as usize] = true;
        self
    }

    pub fn fold2add_f64x4(mut self, xmm_dst: u8, ymm_src: u8) -> Assembly {
        assert!(xmm_dst != ymm_src);

        self.arr.push(Instruction::ExtractU2F64x4(xmm_dst, ymm_src));
        self.arr
            .push(Instruction::AddF64x2(xmm_dst, xmm_dst, ymm_src));
        self.zmm_used[xmm_dst as usize] = true;
        self
    }

    pub fn fold1add_f64x2(mut self, xmm_dst: u8, xmm_src: u8) -> Assembly {
        self.arr.push(Instruction::Fold1AddF64x2(xmm_dst, xmm_src));
        self.zmm_used[xmm_dst as usize] = true;
        self
    }
}
