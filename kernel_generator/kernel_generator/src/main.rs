extern crate core;
extern crate template;

mod argument;

use argument::{ArgumentError, GeneratorType};
use core::{Generate, GenerateError};
use template::*;

const HELP_TEXT: &'static str = "\
Usage:
kernel-generator spmv \
<col_pft> <col_pfd> <col_pld> <val_pft> <val_pfd> <val_pld> <x_pld> \
<rowblock> <nops> <store_to_tmp> <move_reg> <move_base>
or
kernel-generator trsv \
<direction> <static_iter> \
<col_pft> <col_pfd> <col_pld> <val_pft> <val_pfd> <val_pld> <x_pld> \
<rowblock> <nops> <store_to_tmp> <move_reg> <move_base>
or
kernel-generator symgs \
<static_iter> \
<col_pft> <col_pfd> <col_pld> <val_pft> <val_pfd> <val_pld> <x_pld> \
<spmv_rowblock> <sptrsv_rowblock> <nops_c> <nops_f0> <nops_f1> <nops_f2> <nops_b0> <nops_b1> <nops_b2> \
<store_to_tmp> <move_reg> <move_base>";

// spmv
//
// <col_pft> : column prefetch type [T0, T1, **T2**, NTA, None]
// <col_pfd> : column prefetch distance (integer > 0, **4096**)
// <col_pld> : column preload distance [0, **1**, 2, ...]
//
// <val_pft> : value prefetch type [T0, T1, **T2**, NTA, None]
// <val_pfd> : value prefetch distance (integer > 0, **4096**)
// <val_pld> : value preload distance [**-1**, 0, 1, 2, ...] (-1 : fused load-add for value data)
//
// <x_pld> : xv preload distance [0, 1, **2**, ...]
//
// <rowblock> : rowblock size [1, 2, 4, **8**]
// <nops> : # of nops [0, 1, ...]
// <store_to_tmp> : store temporary rowblock result to memory (**f**, t)
// <move_reg> : move data on registers for preloading instead of unrolling (f, **t**)
// <move_base> : move base inside of nanokernel (f, **t**)

// trsv
// 
// <direction> : forward / backward (f, b)
// <static_iter> : additional pre/post trsv that iterates constant time for wavefront (0, 1, 2, ...)
//
// <col_pft> : column prefetch type [T0, T1, **T2**, NTA, None]
// <col_pfd> : column prefetch distance (integer > 0, **4096**)
// <col_pld> : column preload distance [0, **1**, 2, ...]
//
// <val_pft> : value prefetch type [T0, T1, **T2**, NTA, None]
// <val_pfd> : value prefetch distance (integer > 0, **4096**)
// <val_pld> : value preload distance [**-1**, 0, 1, 2, ...] (-1 : fused load-add for value data)
//
// <x_pld> : xv preload distance [0, 1, **2**, ...]
//
// <rowblock> : rowblock size [1, 2, 4, **8**]
// <nops> : # of nops [0, 1, ...]
// <store_to_tmp> : store temporary rowblock result to memory (**f**, t)
// <move_reg> : move data on registers for preloading instead of unrolling (f, **t**)
// <move_base> : move base inside of nanokernel (f, **t**)

// symgs
// 
// <static_iter> : additional pre/post trsv that iterates constant time for wavefront (0, 1, 2, ...)
//
// <col_pft> : column prefetch type [T0, T1, **T2**, NTA, None]
// <col_pfd> : column prefetch distance (integer > 0, **4096**)
// <col_pld> : column preload distance [0, **1**, 2, ...]
//
// <val_pft> : value prefetch type [T0, T1, **T2**, NTA, None]
// <val_pfd> : value prefetch distance (integer > 0, **4096**)
// <val_pld> : value preload distance [**-1**, 0, 1, 2, ...] (-1 : fused load-add for value data)
//
// <x_pld> : xv preload distance [0, 1, **2**, ...]
//
// <spmv_rowblock> : rowblock size for precomputing spmv [1, 2, 4, **8**]
// <trsv_rowblock> : rowblock size for forward/backward trsv [1, 2, 4, **8**]
// <nops_c> : # of nops for precomputing spmv [0, 1, ...]
// <nops_f0> : # of nops for preforwarding trsv [0, 1, ...]
// <nops_f1> : # of nops for forwarding trsv [0, 1, ...]
// <nops_f2> : # of nops for postforwarding trsv [0, 1, ...]
// <nops_b0> : # of nops for prebackwarding trsv [0, 1, ...]
// <nops_b1> : # of nops for backwarding trsv [0, 1, ...]
// <nops_b2> : # of nops for postbackwarding trsv [0, 1, ...]
//
// <store_to_tmp> : store temporary rowblock result to memory (**f**, t)
// <move_reg> : move data on registers for preloading instead of unrolling (f, **t**)
// <move_base> : move base inside of nanokernel (f, **t**)

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let generator = argument::parse_arguments(&args[1..]).unwrap_or_else(|err| {
        match err {
            ArgumentError::InvalidArgument => {
                eprintln!("Error: invalid argument");
                eprintln!("{}", HELP_TEXT);
            }
            ArgumentError::NotEnoughArguments => {
                eprintln!("Error: not enough arguments");
                eprintln!("{}", HELP_TEXT);
            }
            ArgumentError::TooManyArguments => {
                eprintln!("Error: too many arguments");
                eprintln!("{}", HELP_TEXT);
            }
        }
        std::process::exit(1);
    });

    let code = match generator {
        GeneratorType::Spmv(spmv_generator) => get_spmv_code(spmv_generator),
        GeneratorType::Sptrsv(sptrsv_generator) => get_sptrsv_code(sptrsv_generator),
        GeneratorType::Symgs(symgs_generator) => get_symgs_code(symgs_generator),
    };

    let code = code.unwrap_or_else(|err| {
        match err {
            GenerateError::RegisterOverflow => eprintln!("Error: somehow register overflowed."),
            GenerateError::IllegalUnrollFactor => {
                eprintln!("Error: somehow used illegal unroll factor.")
            }
        }
        std::process::exit(1);
    });

    println!("{}", code);
}

fn get_spmv_code(spmv_generator: spmv::Generator) -> Result<String, GenerateError> {
    let header_code = "\
    extern \"C\" int spmv(\
    int nrow, \
    const int* col, \
    const double* val, \
    const double* x, \
    double* tmp, \
    double* y) { \n    \
    int i;\n\n";
    let tail_code = "\n    return 0;\n}\n";

    let asm = spmv_generator.generate()?;

    let variable_names = ["nrow", "col", "x", "val", "tmp", "y", "i"];
    let asm_names = ["NROW", "COL", "X", "VAL", "TMP", "Y", "I"];
    let main_code = asm.print(1, &variable_names, &asm_names);

    Ok(format!("{}{}{}", header_code, main_code, tail_code))
}

fn get_sptrsv_code(sptrsv_generator: sptrsv::Generator) -> Result<String, GenerateError> {
    let header_code = "\
    extern \"C\" int sptrsv(\
    int nrow, \
    const int* col, \
    const double* val, \
    double* x, \
    double* tmp, \
    double* p, \
    const double* d, \
    const double* r) {\n    \
    int i;\n    \
    double* imm_x = x;\n\n";
    let tail_code = "\n    return 0;\n}\n";

    let asm = sptrsv_generator.generate()?;

    let variable_names = [
        "nrow", "col", "val", "x", "imm_x", "tmp", "i", "p", "d", "r",
    ];
    let asm_names = [
        "NROW", "COL", "VAL", "X", "IMM_X", "TMP", "I", "P", "D", "R",
    ];
    let main_code = asm.print(1, &variable_names, &asm_names);

    Ok(format!("{}{}{}", header_code, main_code, tail_code))
}

fn get_symgs_code(symgs_generator: symgs::Generator) -> Result<String, GenerateError> {
    let header_code = "\
    extern \"C\" int symgs(\
    int nrow, \
    const int* ucol, \
    const int* lcol, \
    const double* uval, \
    const double* lval, \
    double* x, \
    double* tmp, \
    double* p, \
    const double* d, \
    const double* r) {\n    \
    int i;\n    \
    int imm_nrow = nrow;\n    \
    double* imm_x = x;\n    \
    double* imm_p = p;\n\n";
    let tail_code = "\n    return 0;\n}\n";

    let asm = symgs_generator.generate()?;

    let variable_names = [
        "nrow", "imm_nrow", "ucol", "lcol", "uval", "lval", "x", "imm_x", "tmp", "i", "p", "imm_p",
        "d", "r",
    ];
    let asm_names = [
        "NROW", "IMM_NROW", "UCOL", "LCOL", "UVAL", "LVAL", "X", "IMM_X", "TMP", "I", "P", "IMM_P",
        "D", "R",
    ];
    let main_code = asm.print(1, &variable_names, &asm_names);

    Ok(format!("{}{}{}", header_code, main_code, tail_code))
}
