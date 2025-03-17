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
        lcol_name: &'static str,

        val_prefetch_info: Option<(PrefetchType, u16)>,
        val_preload_dist: Option<u8>,
        lval_name: &'static str,

        x_preload_dist: u8,
        x_name: &'static str,
        immutable_x_name: &'static str,

        tmp_name: &'static str,

        cnt_name: &'static str,
        preforwarding_loop_name: &'static str,
        forwarding_loop_name: &'static str,
        postforwarding_loop_name: &'static str,

        p_name: &'static str,
        d_name: &'static str,
        r_name: &'static str,

        rowblock_size: u8,
        nops_before_preforwarding: u8,
        nops_before_forwarding: u8,
        nops_before_postforwarding: u8,
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
            lcol_name,
            val_prefetch_info,
            val_preload_dist,
            lval_name,
            x_preload_dist,
            x_name,
            immutable_x_name,
            tmp_name,
            cnt_name,
            preforwarding_loop_name,
            forwarding_loop_name,
            postforwarding_loop_name,
            p_name,
            d_name,
            r_name,
            rowblock_size,
            nops_before_preforwarding,
            nops_before_forwarding,
            nops_before_postforwarding,
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
    lcol_name: &'static str,

    val_prefetch_info: Option<(PrefetchType, u16)>,
    val_preload_dist: Option<u8>,
    lval_name: &'static str,

    x_preload_dist: u8,
    x_name: &'static str,
    immutable_x_name: &'static str,

    tmp_name: &'static str,

    cnt_name: &'static str,
    preforwarding_loop_name: &'static str,
    forwarding_loop_name: &'static str,
    postforwarding_loop_name: &'static str,

    p_name: &'static str,
    d_name: &'static str,
    r_name: &'static str,

    rowblock_size: u8,
    nops_before_preforwarding: u8,
    nops_before_forwarding: u8,
    nops_before_postforwarding: u8,
    store_to_tmp: bool,
    move_reg: bool,
    move_base: bool,
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
                Direction::Forward,
                Some(config.static_iter),
                config.nrow_name,
                0,
                None,
                config.col_preload_dist,
                config.lcol_name,
                0,
                None,
                config.val_preload_dist,
                config.lval_name,
                config.x_preload_dist,
                config.x_name,
                config.immutable_x_name,
                config.tmp_name,
                config.cnt_name,
                config.preforwarding_loop_name,
                config.p_name,
                config.d_name,
                config.r_name,
                1,
                config.nops_before_preforwarding,
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
                Direction::Forward,
                None,
                config.nrow_name,
                0,
                config.col_prefetch_info,
                config.col_preload_dist,
                config.lcol_name,
                0,
                config.val_prefetch_info,
                config.val_preload_dist,
                config.lval_name,
                config.x_preload_dist,
                config.x_name,
                config.immutable_x_name,
                config.tmp_name,
                config.cnt_name,
                config.forwarding_loop_name,
                config.p_name,
                config.d_name,
                config.r_name,
                config.rowblock_size,
                config.nops_before_forwarding,
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
                Direction::Forward,
                Some(config.static_iter),
                config.nrow_name,
                0,
                None,
                config.col_preload_dist,
                config.lcol_name,
                0,
                None,
                config.val_preload_dist,
                config.lval_name,
                config.x_preload_dist,
                config.x_name,
                config.immutable_x_name,
                config.tmp_name,
                config.cnt_name,
                config.postforwarding_loop_name,
                config.p_name,
                config.d_name,
                config.r_name,
                1,
                config.nops_before_postforwarding,
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
