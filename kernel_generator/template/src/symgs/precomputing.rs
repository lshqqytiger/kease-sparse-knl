use crate::spmv;
use crate::Direction;
use core::*;

impl Generator {
    pub fn new(
        matrix_format: sparse_matrix::SparseMatrixFormat,

        nrow_name: &'static str,

        col_prefetch_info: Option<(PrefetchType, u16)>,
        col_preload_dist: u8,
        ucol_name: &'static str,

        val_prefetch_info: Option<(PrefetchType, u16)>,
        val_preload_dist: Option<u8>,
        uval_name: &'static str,

        x_preload_dist: u8,
        x_name: &'static str,

        tmp_name: &'static str,

        cnt_name: &'static str,
        loop_name: &'static str,

        p_name: &'static str,
        immutable_p_name: &'static str,

        rowblock_size: u8,
        nops_before_precomputing: u8,
        store_to_tmp: bool,
        move_reg: bool,
        move_base: bool,
    ) -> Self {
        Generator {
            matrix_format,
            nrow_name,
            col_prefetch_info,
            col_preload_dist,
            ucol_name,
            val_prefetch_info,
            val_preload_dist,
            uval_name,
            x_preload_dist,
            x_name,
            tmp_name,
            cnt_name,
            loop_name,
            p_name,
            immutable_p_name,
            rowblock_size,
            nops_before_precomputing,
            store_to_tmp,
            move_reg,
            move_base,
        }
    }
}

pub struct Generator {
    matrix_format: sparse_matrix::SparseMatrixFormat,

    nrow_name: &'static str,

    col_prefetch_info: Option<(PrefetchType, u16)>,
    col_preload_dist: u8,
    ucol_name: &'static str,

    val_prefetch_info: Option<(PrefetchType, u16)>,
    val_preload_dist: Option<u8>,
    uval_name: &'static str,

    x_preload_dist: u8,
    x_name: &'static str,

    tmp_name: &'static str,

    cnt_name: &'static str,
    loop_name: &'static str,

    p_name: &'static str,
    immutable_p_name: &'static str,

    rowblock_size: u8,
    nops_before_precomputing: u8,
    store_to_tmp: bool,
    move_reg: bool,
    move_base: bool,
}

enum StateType {
    GeneratingSpmv,
    RestoringP,
}

const RULEBOOK: &'static [Rule<Generator>] = &[
    Rule {
        condition: Condition::Single {
            id: StateType::GeneratingSpmv as u32,
        },
        callback: |config: &Generator, _rp: &mut RegisterPool, _states: &Vec<State>| {
            let action = spmv::Action::AssignNegUx;
            let direction = Direction::Forward;

            let spmv_generator = spmv::Generator::new(
                config.matrix_format,
                action,
                direction,
                config.nrow_name,
                config.col_prefetch_info,
                config.col_preload_dist,
                config.ucol_name,
                config.val_prefetch_info,
                config.val_preload_dist,
                config.uval_name,
                config.x_preload_dist,
                config.x_name,
                config.tmp_name,
                config.cnt_name,
                config.loop_name,
                config.p_name,
                config.rowblock_size,
                config.nops_before_precomputing,
                config.store_to_tmp,
                config.move_reg,
                config.move_base,
            );

            let asm = spmv_generator.generate()?;
            let next_id = StateType::RestoringP as u32;
            let states = Vec::from([State {
                id: next_id,
                idx: 0,
                reg: 0,
            }]);

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::Single {
            id: StateType::RestoringP as u32,
        },
        callback: |config: &Generator, _rp: &mut RegisterPool, _states: &Vec<State>| {
            let asm = Assembly::new().move_reg(config.p_name, config.immutable_p_name);
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
        let states = Vec::from([State {
            id: StateType::GeneratingSpmv as u32,
            idx: 0,
            reg: 0,
        }]);

        states
    }
}
