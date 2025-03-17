use crate::{sptrsv, Direction};
use core::sparse_matrix::*;
use core::*;

impl Generator {
    pub fn new(
        matrix_format: SparseMatrixFormat,
        static_iter: Option<u8>,

        nrow_name: &'static str,
        immutable_nrow_name: &'static str,

        col_prefetch_info: Option<(PrefetchType, u16)>,
        col_preload_dist: u8,
        ucol_name: &'static str,

        val_prefetch_info: Option<(PrefetchType, u16)>,
        val_preload_dist: Option<u8>,
        uval_name: &'static str,

        x_preload_dist: u8,
        x_name: &'static str,
        immutable_x_name: &'static str,

        tmp_name: &'static str,

        cnt_name: &'static str,
        prebackwarding_loop_name: &'static str,
        backwarding_loop_name: &'static str,
        postbackwarding_loop_name: &'static str,

        p_name: &'static str,
        d_name: &'static str,
        r_name: &'static str,

        rowblock_size: u8,
        nops_before_prebackwarding: u8,
        nops_before_backwarding: u8,
        nops_before_postbackwarding: u8,
        store_to_tmp: bool,
        move_reg: bool,
        move_base: bool,
    ) -> Self {
        let static_iter = static_iter.unwrap_or(0);

        Generator {
            matrix_format,
            static_iter,
            nrow_name,
            immutable_nrow_name,
            col_prefetch_info,
            col_preload_dist,
            ucol_name,
            val_prefetch_info,
            val_preload_dist,
            uval_name,
            x_preload_dist,
            x_name,
            immutable_x_name,
            tmp_name,
            cnt_name,
            prebackwarding_loop_name,
            backwarding_loop_name,
            postbackwarding_loop_name,
            p_name,
            d_name,
            r_name,
            rowblock_size,
            nops_before_prebackwarding,
            nops_before_backwarding,
            nops_before_postbackwarding,
            store_to_tmp,
            move_reg,
            move_base,
        }
    }
}

pub struct Generator {
    matrix_format: SparseMatrixFormat,
    static_iter: u8,

    nrow_name: &'static str,
    immutable_nrow_name: &'static str,

    col_prefetch_info: Option<(PrefetchType, u16)>,
    col_preload_dist: u8,
    ucol_name: &'static str,

    val_prefetch_info: Option<(PrefetchType, u16)>,
    val_preload_dist: Option<u8>,
    uval_name: &'static str,

    x_preload_dist: u8,
    x_name: &'static str,
    immutable_x_name: &'static str,

    tmp_name: &'static str,

    cnt_name: &'static str,
    prebackwarding_loop_name: &'static str,
    backwarding_loop_name: &'static str,
    postbackwarding_loop_name: &'static str,

    p_name: &'static str,
    d_name: &'static str,
    r_name: &'static str,

    rowblock_size: u8,
    nops_before_prebackwarding: u8,
    nops_before_backwarding: u8,
    nops_before_postbackwarding: u8,
    store_to_tmp: bool,
    move_reg: bool,
    move_base: bool,
}

impl Generator {
    fn lu_default(&self) -> bool {
        match self.matrix_format {
            SparseMatrixFormat::ELL(ell_info) => match ell_info.lu {
                LUStatus::Default => true,
                LUStatus::Excluded => false,
            },
        }
    }

    fn col_stride(&self) -> u16 {
        let stride = match self.lu_default() {
            true => 32,
            false => 16,
        };
        stride * size_of::<i32>() as u16
    }

    fn val_stride(&self) -> u16 {
        let stride = match self.lu_default() {
            true => 32,
            false => 16,
        };
        stride * size_of::<f64>() as u16
    }

    fn col_premove(&self) -> i16 {
        self.col_stride() as i16 * self.col_preload_dist as i16 * -1
    }

    fn val_premove(&self) -> i16 {
        self.val_stride() as i16 * self.val_preload_dist.unwrap_or(0) as i16 * -1
    }

    fn prekernel_col_premove(&self) -> i16 {
        match self.static_iter {
            0 => 0,
            _ => self.col_premove(),
        }
    }

    fn prekernel_val_premove(&self) -> i16 {
        match self.static_iter {
            0 => 0,
            _ => self.val_premove(),
        }
    }

    fn kernel_col_premove(&self) -> i16 {
        match self.static_iter {
            0 => self.col_premove(),
            _ => 0,
        }
    }

    fn kernel_val_premove(&self) -> i16 {
        match self.static_iter {
            0 => self.val_premove(),
            _ => 0,
        }
    }
}

enum StateType {
    GeneratingPreSptrsv,
    InitializingNrow,
    GeneratingSptrsv,
    GeneratingPostSptrsv,
}

const RULEBOOK: &'static [Rule<Generator>] = &[
    Rule {
        condition: Condition::Single {
            id: StateType::GeneratingPreSptrsv as u32,
        },
        callback: |config: &Generator, _rp: &mut RegisterPool, _states: &Vec<State>| {
            let sptrsv_generator = sptrsv::Generator::new(
                config.matrix_format,
                Direction::Backward,
                Some(config.static_iter),
                config.nrow_name,
                config.prekernel_col_premove(),
                None,
                config.col_preload_dist,
                config.ucol_name,
                config.prekernel_val_premove(),
                None,
                config.val_preload_dist,
                config.uval_name,
                config.x_preload_dist,
                config.x_name,
                config.immutable_x_name,
                config.tmp_name,
                config.cnt_name,
                config.prebackwarding_loop_name,
                config.p_name,
                config.d_name,
                config.r_name,
                1,
                config.nops_before_prebackwarding,
                false,
                config.move_reg,
                config.move_base,
                false,
            );

            let asm = sptrsv_generator.generate()?.empty_line();
            let states = Vec::new();

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::Single {
            id: StateType::InitializingNrow as u32,
        },
        callback: |config: &Generator, _rp: &mut RegisterPool, _states: &Vec<State>| {
            let asm = {
                let asm_move =
                    Assembly::new().move_reg(config.nrow_name, config.immutable_nrow_name);
                let asm_sub = match config.static_iter {
                    0 => Assembly::new(),
                    x => Assembly::new().sub_immediate(config.nrow_name, x as i16 * 2),
                };

                asm_move.append(asm_sub)
            }
            .empty_line();
            let states = Vec::new();

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::Single {
            id: StateType::GeneratingSptrsv as u32,
        },
        callback: |config: &Generator, _rp: &mut RegisterPool, _states: &Vec<State>| {
            let skip_preload = config.static_iter > 1;

            let sptrsv_generator = sptrsv::Generator::new(
                config.matrix_format,
                Direction::Backward,
                None,
                config.nrow_name,
                config.kernel_col_premove(),
                config.col_prefetch_info,
                config.col_preload_dist,
                config.ucol_name,
                config.kernel_val_premove(),
                config.val_prefetch_info,
                config.val_preload_dist,
                config.uval_name,
                config.x_preload_dist,
                config.x_name,
                config.immutable_x_name,
                config.tmp_name,
                config.cnt_name,
                config.backwarding_loop_name,
                config.p_name,
                config.d_name,
                config.r_name,
                config.rowblock_size,
                config.nops_before_backwarding,
                config.store_to_tmp,
                config.move_reg,
                config.move_base,
                skip_preload,
            );

            let asm = sptrsv_generator.generate()?.empty_line();
            let states = Vec::new();

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::Single {
            id: StateType::GeneratingPostSptrsv as u32,
        },
        callback: |config: &Generator, _rp: &mut RegisterPool, _states: &Vec<State>| {
            let sptrsv_generator = sptrsv::Generator::new(
                config.matrix_format,
                Direction::Backward,
                Some(config.static_iter),
                config.nrow_name,
                0,
                None,
                config.col_preload_dist,
                config.ucol_name,
                0,
                None,
                config.val_preload_dist,
                config.uval_name,
                config.x_preload_dist,
                config.x_name,
                config.immutable_x_name,
                config.tmp_name,
                config.cnt_name,
                config.postbackwarding_loop_name,
                config.p_name,
                config.d_name,
                config.r_name,
                1,
                config.nops_before_postbackwarding,
                false,
                config.move_reg,
                config.move_base,
                true,
            );

            let asm = sptrsv_generator.generate()?;
            let states = Vec::new();

            Ok((asm, states))
        },
    },
];

impl Generate for Generator {
    fn rulebook(&self) -> &'static [Rule<Self>] {
        RULEBOOK
    }

    fn avail_registers(&self) -> [bool; 32] {
        [false; 32]
    }

    fn initial_states(&self) -> Vec<State> {
        let mut states = Vec::new();

        if self.static_iter > 0 {
            states.push(State {
                id: StateType::GeneratingPreSptrsv as u32,
                idx: 0,
                reg: 0,
            });
        }

        states.push(State {
            id: StateType::InitializingNrow as u32,
            idx: 0,
            reg: 0,
        });

        states.push(State {
            id: StateType::GeneratingSptrsv as u32,
            idx: 0,
            reg: 0,
        });

        if self.static_iter > 0 {
            states.push(State {
                id: StateType::GeneratingPostSptrsv as u32,
                idx: 0,
                reg: 0,
            });
        }

        states
    }
}
