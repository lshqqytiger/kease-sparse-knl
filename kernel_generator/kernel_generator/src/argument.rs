use core::{sparse_matrix, PrefetchType};
use template::*;

use std::fmt;

pub enum GeneratorType {
    Spmv(spmv::Generator),
    Sptrsv(sptrsv::Generator),
    Symgs(symgs::Generator),
}

pub fn parse_arguments(args: &[String]) -> Result<GeneratorType, ArgumentError> {
    if args.len() == 0 {
        return Err(ArgumentError::InvalidArgument);
    }

    match args[0].as_str() {
        "spmv" | "SPMV" => parse_spmv_arguments(&args[1..]).map(|g| GeneratorType::Spmv(g)),
        "trsv" | "TRSV" | "sptrsv" | "SPTRSV" => {
            parse_sptrsv_arguments(&args[1..]).map(|g| GeneratorType::Sptrsv(g))
        }
        "symgs" | "SYMGS" => parse_symgs_arguments(&args[1..]).map(|g| GeneratorType::Symgs(g)),
        _ => Err(ArgumentError::InvalidArgument),
    }
}

fn parse_spmv_arguments(args: &[String]) -> Result<spmv::Generator, ArgumentError> {
    let mut iter = args.iter();
    let mut next = || {
        iter.next()
            .map_or(Err(ArgumentError::NotEnoughArguments), |s| Ok(s))
    };

    let matrix_format = {
        // TODO: support ELL_Info
        let ell_info = sparse_matrix::ELLInfo::new(
            sparse_matrix::DiagonalStatus::Default,
            sparse_matrix::LUStatus::Default,
            sparse_matrix::GridPointOrdering::Default,
        );
        sparse_matrix::SparseMatrixFormat::ELL(ell_info)
    };
    let action = spmv::Action::AssignPosAx;
    let direction = Direction::Forward;

    let col_prefetch_info = ArgumentParser::parse_prefetch_info(next()?, next()?)?;
    let col_preload_dist = ArgumentParser::parse_u8(next()?)?;

    let val_prefetch_info = ArgumentParser::parse_prefetch_info(next()?, next()?)?;
    let val_preload_dist = ArgumentParser::parse_option_u8(next()?)?;

    let x_preload_dist = ArgumentParser::parse_u8(next()?)?;

    let rowblock_size = ArgumentParser::parse_u8(next()?)?;
    let n_nops = ArgumentParser::parse_u8(next()?)?;
    let store_to_tmp = ArgumentParser::parse_bool(next()?)?;
    let move_reg = ArgumentParser::parse_bool(next()?)?;
    let move_base = ArgumentParser::parse_bool(next()?)?;

    if next().is_ok() {
        return Err(ArgumentError::TooManyArguments);
    }

    let spmv_generator = spmv::Generator::new(
        matrix_format,
        action,
        direction,
        "NROW",
        col_prefetch_info,
        col_preload_dist,
        "COL",
        val_prefetch_info,
        val_preload_dist,
        "VAL",
        x_preload_dist,
        "X",
        "TMP",
        "I",
        "loop_spmv",
        "Y",
        rowblock_size,
        n_nops,
        store_to_tmp,
        move_reg,
        move_base,
    );

    Ok(spmv_generator)
}

fn parse_sptrsv_arguments(args: &[String]) -> Result<sptrsv::Generator, ArgumentError> {
    let mut iter = args.iter();
    let mut next = || {
        iter.next()
            .map_or(Err(ArgumentError::NotEnoughArguments), |s| Ok(s))
    };

    let matrix_format = {
        // TODO: support ELL_Info
        let ell_info = sparse_matrix::ELLInfo::new(
            sparse_matrix::DiagonalStatus::Excluded,
            sparse_matrix::LUStatus::Default,
            sparse_matrix::GridPointOrdering::Default,
        );
        sparse_matrix::SparseMatrixFormat::ELL(ell_info)
    };

    let direction = ArgumentParser::parse_direction(next()?)?;
    let static_iter = ArgumentParser::parse_option_u8(next()?)?;

    let col_prefetch_info = ArgumentParser::parse_prefetch_info(next()?, next()?)?;
    let col_preload_dist = ArgumentParser::parse_u8(next()?)?;

    let val_prefetch_info = ArgumentParser::parse_prefetch_info(next()?, next()?)?;
    let val_preload_dist = ArgumentParser::parse_option_u8(next()?)?;

    let x_preload_dist = ArgumentParser::parse_u8(next()?)?;

    let rowblock_size = ArgumentParser::parse_u8(next()?)?;
    let n_nops = ArgumentParser::parse_u8(next()?)?;
    let store_to_tmp = ArgumentParser::parse_bool(next()?)?;
    let move_reg = ArgumentParser::parse_bool(next()?)?;
    let move_base = ArgumentParser::parse_bool(next()?)?;

    if next().is_ok() {
        return Err(ArgumentError::TooManyArguments);
    }

    let sptrsv_generator = sptrsv::Generator::new(
        matrix_format,
        direction,
        static_iter,
        "NROW",
        0,
        col_prefetch_info,
        col_preload_dist,
        "COL",
        0,
        val_prefetch_info,
        val_preload_dist,
        "VAL",
        x_preload_dist,
        "X",
        "IMM_X",
        "TMP",
        "I",
        "loop_sptrsv",
        "P",
        "D",
        "R",
        rowblock_size,
        n_nops,
        store_to_tmp,
        move_reg,
        move_base,
        false,
    );

    Ok(sptrsv_generator)
}

fn parse_symgs_arguments(args: &[String]) -> Result<symgs::Generator, ArgumentError> {
    let mut iter = args.iter();
    let mut next = || {
        iter.next()
            .map_or(Err(ArgumentError::NotEnoughArguments), |s| Ok(s))
    };

    let matrix_format = {
        // TODO: support ELL_Info
        let ell_info = sparse_matrix::ELLInfo::new(
            sparse_matrix::DiagonalStatus::Excluded,
            sparse_matrix::LUStatus::Default,
            sparse_matrix::GridPointOrdering::Default,
        );
        sparse_matrix::SparseMatrixFormat::ELL(ell_info)
    };

    let sptrsv_static_iter = ArgumentParser::parse_option_u8(next()?)?;

    let col_prefetch_info = ArgumentParser::parse_prefetch_info(next()?, next()?)?;
    let col_preload_dist = ArgumentParser::parse_u8(next()?)?;

    let val_prefetch_info = ArgumentParser::parse_prefetch_info(next()?, next()?)?;
    let val_preload_dist = ArgumentParser::parse_option_u8(next()?)?;

    let x_preload_dist = ArgumentParser::parse_u8(next()?)?;

    let spmv_rowblock_size = ArgumentParser::parse_u8(next()?)?;
    let sptrsv_rowblock_size = ArgumentParser::parse_u8(next()?)?;
    let n_nops_c = ArgumentParser::parse_u8(next()?)?;
    let n_nops_f0 = ArgumentParser::parse_u8(next()?)?;
    let n_nops_f1 = ArgumentParser::parse_u8(next()?)?;
    let n_nops_f2 = ArgumentParser::parse_u8(next()?)?;
    let n_nops_b0 = ArgumentParser::parse_u8(next()?)?;
    let n_nops_b1 = ArgumentParser::parse_u8(next()?)?;
    let n_nops_b2 = ArgumentParser::parse_u8(next()?)?;
    let store_to_tmp = ArgumentParser::parse_bool(next()?)?;
    let move_reg = ArgumentParser::parse_bool(next()?)?;
    let move_base = ArgumentParser::parse_bool(next()?)?;

    if next().is_ok() {
        return Err(ArgumentError::TooManyArguments);
    }

    let symgs_generator = symgs::Generator::new(
        matrix_format,
        sptrsv_static_iter,
        "NROW",
        "IMM_NROW",
        col_prefetch_info,
        col_preload_dist,
        "UCOL",
        "LCOL",
        val_prefetch_info,
        val_preload_dist,
        "UVAL",
        "LVAL",
        x_preload_dist,
        "X",
        "IMM_X",
        "TMP",
        "I",
        "loop_c",
        "loop_f0",
        "loop_f1",
        "loop_f2",
        "loop_b0",
        "loop_b1",
        "loop_b2",
        "P",
        "IMM_P",
        "D",
        "R",
        spmv_rowblock_size,
        sptrsv_rowblock_size,
        n_nops_c,
        n_nops_f0,
        n_nops_f1,
        n_nops_f2,
        n_nops_b0,
        n_nops_b1,
        n_nops_b2,
        store_to_tmp,
        move_reg,
        move_base,
    );

    Ok(symgs_generator)
}

struct ArgumentParser;

impl ArgumentParser {
    fn parse_prefetch_info(
        arg0: &str,
        arg1: &str,
    ) -> Result<Option<(PrefetchType, u16)>, ArgumentError> {
        let prefetch_kind = match arg0 {
            "T0" | "t0" => Ok(Some(PrefetchType::T0)),
            "T1" | "t1" => Ok(Some(PrefetchType::T1)),
            "T2" | "t2" => Ok(Some(PrefetchType::T2)),
            "NTA" | "nta" => Ok(Some(PrefetchType::NTA)),
            "None" | "none" | "n" => Ok(None),
            _ => Err(ArgumentError::InvalidArgument),
        }?;

        let prefetch_dist = arg1
            .parse::<u16>()
            .map_err(|_| ArgumentError::InvalidArgument)?;

        Ok(prefetch_kind.map(|kind| (kind, prefetch_dist)))
    }

    fn parse_u8(arg: &str) -> Result<u8, ArgumentError> {
        arg.parse::<u8>()
            .map_err(|_| ArgumentError::InvalidArgument)
    }

    fn parse_option_u8(arg: &str) -> Result<Option<u8>, ArgumentError> {
        let val = arg
            .parse::<i16>()
            .map_err(|_| ArgumentError::InvalidArgument)?;

        match val {
            -1 => Ok(None),
            0..256 => Ok(Some(val as u8)),
            _ => Err(ArgumentError::InvalidArgument),
        }
    }

    fn parse_direction(arg: &str) -> Result<Direction, ArgumentError> {
        match arg {
            "F" | "f" | "forward" | "Forward" | "FORWARD" => Ok(Direction::Forward),
            "B" | "b" | "backward" | "Backward" | "BACKWARD" => Ok(Direction::Backward),
            _ => Err(ArgumentError::InvalidArgument),
        }
    }

    fn parse_bool(arg: &str) -> Result<bool, ArgumentError> {
        match arg {
            "T" | "t" | "true" | "True" | "TRUE" => Ok(true),
            "F" | "f" | "false" | "False" | "FALSE" => Ok(false),
            _ => Err(ArgumentError::InvalidArgument),
        }
    }
}

#[derive(Debug)]
pub enum ArgumentError {
    InvalidArgument,
    NotEnoughArguments,
    TooManyArguments,
}

impl fmt::Display for ArgumentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArgument => write!(f, "invalid argument"),
            Self::NotEnoughArguments => write!(f, "not enough argument"),
            Self::TooManyArguments => write!(f, "too many arguments"),
        }
    }
}

impl std::error::Error for ArgumentError {}
