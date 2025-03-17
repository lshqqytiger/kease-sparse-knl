// symgs implementation method : Xiaojian Yang, Shengguo Li, Fan Yuan, Dezun Dong, Chun Huang, and Zheng Wang. 2023. Optimizing Multi-grid Computation and Parallelization on Multi-cores. In Proceedings of the 37th ACM International Conference on Supercomputing (ICS '23). Association for Computing Machinery, New York, NY, USA, 227–239. https://doi.org/10.1145/3577193.3593726

use core::sparse_matrix::*;
use core::*;

mod backwarding;
mod forwarding;
mod precomputing;

impl Generator {
    pub fn new(
        matrix_format: SparseMatrixFormat,
        sptrsv_static_iter: Option<u8>,

        nrow_name: &'static str,
        immutable_nrow_name: &'static str,

        col_prefetch_info: Option<(PrefetchType, u16)>,
        col_preload_dist: u8,
        ucol_name: &'static str,
        lcol_name: &'static str,

        val_prefetch_info: Option<(PrefetchType, u16)>,
        val_preload_dist: Option<u8>,
        uval_name: &'static str,
        lval_name: &'static str,

        x_preload_dist: u8,
        x_name: &'static str,
        immutable_x_name: &'static str,

        tmp_name: &'static str,

        cnt_name: &'static str,
        precomputing_loop_name: &'static str,
        preforwarding_loop_name: &'static str,
        forwarding_loop_name: &'static str,
        postforwarding_loop_name: &'static str,
        prebackwarding_loop_name: &'static str,
        backwarding_loop_name: &'static str,
        postbackwarding_loop_name: &'static str,

        p_name: &'static str,
        immutable_p_name: &'static str,
        d_name: &'static str,
        r_name: &'static str,

        spmv_rowblock_size: u8,
        sptrsv_rowblock_size: u8,
        nops_before_precomputing: u8,
        nops_before_preforwarding: u8,
        nops_before_forwarding: u8,
        nops_before_postforwarding: u8,
        nops_before_prebackwarding: u8,
        nops_before_backwarding: u8,
        nops_before_postbackwarding: u8,
        store_to_tmp: bool,
        move_reg: bool,
        move_base: bool,
    ) -> Self {
        let generator = Generator {
            matrix_format,
            sptrsv_static_iter,
            nrow_name,
            immutable_nrow_name,
            col_prefetch_info,
            col_preload_dist,
            ucol_name,
            lcol_name,
            val_prefetch_info,
            val_preload_dist,
            uval_name,
            lval_name,
            x_preload_dist,
            x_name,
            immutable_x_name,
            tmp_name,
            cnt_name,
            precomputing_loop_name,
            preforwarding_loop_name,
            forwarding_loop_name,
            postforwarding_loop_name,
            prebackwarding_loop_name,
            backwarding_loop_name,
            postbackwarding_loop_name,
            p_name,
            immutable_p_name,
            d_name,
            r_name,
            spmv_rowblock_size,
            sptrsv_rowblock_size,
            nops_before_precomputing,
            nops_before_preforwarding,
            nops_before_forwarding,
            nops_before_postforwarding,
            nops_before_prebackwarding,
            nops_before_backwarding,
            nops_before_postbackwarding,
            store_to_tmp,
            move_reg,
            move_base,
        };

        generator
    }
}

pub struct Generator {
    matrix_format: SparseMatrixFormat,
    sptrsv_static_iter: Option<u8>,

    nrow_name: &'static str,
    immutable_nrow_name: &'static str,

    col_prefetch_info: Option<(PrefetchType, u16)>,
    col_preload_dist: u8,
    ucol_name: &'static str,
    lcol_name: &'static str,

    val_prefetch_info: Option<(PrefetchType, u16)>,
    val_preload_dist: Option<u8>,
    uval_name: &'static str,
    lval_name: &'static str,

    x_preload_dist: u8,
    x_name: &'static str,
    immutable_x_name: &'static str,

    tmp_name: &'static str,

    cnt_name: &'static str,
    precomputing_loop_name: &'static str,
    preforwarding_loop_name: &'static str,
    forwarding_loop_name: &'static str,
    postforwarding_loop_name: &'static str,
    prebackwarding_loop_name: &'static str,
    backwarding_loop_name: &'static str,
    postbackwarding_loop_name: &'static str,

    p_name: &'static str,
    immutable_p_name: &'static str,
    d_name: &'static str,
    r_name: &'static str,

    spmv_rowblock_size: u8,
    sptrsv_rowblock_size: u8,
    nops_before_precomputing: u8,
    nops_before_preforwarding: u8,
    nops_before_forwarding: u8,
    nops_before_postforwarding: u8,
    nops_before_prebackwarding: u8,
    nops_before_backwarding: u8,
    nops_before_postbackwarding: u8,
    store_to_tmp: bool,
    move_reg: bool,
    move_base: bool,
}

enum StateType {
    Precomputing, // p = -Ux
    Forwarding,   // x = trsv(D+L, r+p) & p = Dx-p
    Backwarding,  // x = trsv(D+U, p)
}
// +ucol(over), +uval(over), +p

// reset p

// +lcol(over), +lval(over), +x, +p, +r, +d

// fix ucol, uval, x, p, d

// -ucol(over), -uval(over), -x, -p, -d

const RULEBOOK: &'static [Rule<Generator>] = &[
    Rule {
        condition: Condition::Single {
            id: StateType::Precomputing as u32,
        },
        callback: |config: &Generator, _rp: &mut RegisterPool, _states: &Vec<State>| {
            let precomputing_generator = precomputing::Generator::new(
                config.matrix_format,
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
                config.precomputing_loop_name,
                config.p_name,
                config.immutable_p_name,
                config.spmv_rowblock_size,
                config.nops_before_precomputing,
                config.store_to_tmp,
                config.move_reg,
                config.move_base,
            );

            let asm = Assembly::new()
                .comment("--- precomputing start --- //")
                .append(precomputing_generator.generate()?)
                .comment("---  precomputing end  --- //")
                .empty_line();
            let next_id = StateType::Forwarding as u32;
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
            id: StateType::Forwarding as u32,
        },
        callback: |config: &Generator, _rp: &mut RegisterPool, _states: &Vec<State>| {
            let forwarding_generator = forwarding::Generator::new(
                config.matrix_format,
                config.sptrsv_static_iter,
                config.nrow_name,
                config.immutable_nrow_name,
                config.col_prefetch_info,
                config.col_preload_dist,
                config.lcol_name,
                config.val_prefetch_info,
                config.val_preload_dist,
                config.lval_name,
                config.x_preload_dist,
                config.x_name,
                config.immutable_x_name,
                config.tmp_name,
                config.cnt_name,
                config.preforwarding_loop_name,
                config.forwarding_loop_name,
                config.postforwarding_loop_name,
                config.p_name,
                config.d_name,
                config.r_name,
                config.sptrsv_rowblock_size,
                config.nops_before_preforwarding,
                config.nops_before_forwarding,
                config.nops_before_postforwarding,
                config.store_to_tmp,
                config.move_reg,
                config.move_base,
            );

            let asm = Assembly::new()
                .comment("--- forwarding start --- //")
                .append(forwarding_generator.generate()?)
                .comment("---  forwarding end  --- //")
                .empty_line();
            let next_id = StateType::Backwarding as u32;
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
            id: StateType::Backwarding as u32,
        },
        callback: |config: &Generator, _rp: &mut RegisterPool, _states: &Vec<State>| {
            let backwarding_generator = backwarding::Generator::new(
                config.matrix_format,
                config.sptrsv_static_iter,
                config.nrow_name,
                config.immutable_nrow_name,
                config.col_prefetch_info,
                config.col_preload_dist,
                config.ucol_name,
                config.val_prefetch_info,
                config.val_preload_dist,
                config.uval_name,
                config.x_preload_dist,
                config.x_name,
                config.immutable_x_name,
                config.tmp_name,
                config.cnt_name,
                config.prebackwarding_loop_name,
                config.backwarding_loop_name,
                config.postbackwarding_loop_name,
                config.p_name,
                config.d_name,
                config.r_name,
                config.sptrsv_rowblock_size,
                config.nops_before_prebackwarding,
                config.nops_before_backwarding,
                config.nops_before_postbackwarding,
                config.store_to_tmp,
                config.move_reg,
                config.move_base,
            );

            let asm = Assembly::new()
                .comment("--- backwarding start --- //")
                .append(backwarding_generator.generate()?)
                .comment("---  backwarding end  --- //")
                .empty_line();
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
        let initial_state = State {
            id: StateType::Precomputing as u32,
            idx: 0,
            reg: 0,
        };
        let states = Vec::from([initial_state]);

        states
    }
}
