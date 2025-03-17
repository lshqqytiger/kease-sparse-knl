// current = D(0..1), L(1..16), U(16..32)
// need = L(0..16), D(16..17), U(17..32)

use crate::{accumulate, microkernel};
use core::sparse_matrix::*;
use core::*;
use microkernel::{Direction, IterationType};

mod end;

impl Generator {
    pub fn new(
        matrix_format: SparseMatrixFormat,
        direction: Direction,
        static_iter: Option<u8>,

        nrow_name: &'static str,

        col_premove: i16,
        col_prefetch_info: Option<(PrefetchType, u16)>,
        col_preload_dist: u8,
        col_name: &'static str,

        val_premove: i16,
        val_prefetch_info: Option<(PrefetchType, u16)>,
        val_preload_dist: Option<u8>,
        val_name: &'static str,

        x_preload_dist: u8,
        x_name: &'static str,
        immutable_x_name: &'static str,

        tmp_name: &'static str,

        cnt_name: &'static str,
        loop_name: &'static str,

        p_name: &'static str,
        d_name: &'static str,
        r_name: &'static str,

        rowblock_size: u8,
        n_nops: u8,
        store_to_tmp: bool,
        move_reg: bool,
        move_base: bool,
        skip_preload: bool,
    ) -> Self {
        Generator {
            matrix_format,
            direction,
            static_iter,
            nrow_name,
            col_premove,
            col_prefetch_info,
            col_preload_dist,
            col_name,
            val_premove,
            val_prefetch_info,
            val_preload_dist,
            val_name,
            x_preload_dist,
            x_name,
            immutable_x_name,
            tmp_name,
            cnt_name,
            loop_name,
            p_name,
            d_name,
            r_name,
            rowblock_size,
            n_nops,
            store_to_tmp,
            move_reg,
            move_base,
            skip_preload,
        }
        /*
        let blocks_per_row = 2;

        let (col_stride, val_stride) = {
            let stride = match lu_default {
                true => 32,
                false => 16,
            };
            (
                stride * size_of::<i32>() as u16,
                stride * size_of::<f64>() as u16,
            )
        };

        let (col_reg_se, val_reg_se, x_reg_se, res_reg_se) = {
            let col_need = blocks_per_row * (col_preload_dist + 1);
            let val_need = blocks_per_row * val_preload_dist.map_or(0, |d| d + 1);
            let x_need = blocks_per_row * (x_preload_dist + 1);
            let res_need = match store_to_tmp {
                true => 1,
                false => rowblock_size,
            };

            let col_se = (0, col_need);
            let res_se = (col_se.1, col_se.1 + res_need);
            let x_se = (res_se.1, res_se.1 + x_need);
            let val_se = match val_need {
                0 => None,
                _ => Some((x_se.1, x_se.1 + val_need)),
            };

            (col_se, val_se, x_se, res_se)
        };

        let diag_status = match matrix_format {
            SparseMatrixFormat::ELL(ell_info) => ell_info.diag,
        };

        let n_kernels_unrolled = match move_reg {
            true => 1,
            false => {
                let col_groups = col_preload_dist + 1;
                let val_groups = val_preload_dist.map_or(0, |d| d + 1);
                let x_groups = x_preload_dist + 1;
                let res_groups = match store_to_tmp {
                    true => 1,
                    false => rowblock_size,
                };

                let n_regs = [col_groups, val_groups, x_groups, res_groups];

                tools::lcm(&n_regs)
            }
        };
        let kernels_iter = rowblock_size / n_kernels_unrolled;

        if rowblock_size % n_kernels_unrolled != 0 {
            return Err(GenerateError::IllegalUnrollFactor);
        }

        let premove_base = match direction {
            Direction::Forward => false,
            Direction::Backward => true,
        };

        let sptrsv_generator = Generator {
            static_iter,
            nrow_name,
            col_stride,
            col_prefetch_info,
            col_reg_se,
            col_name,
            val_stride,
            val_prefetch_info,
            val_reg_se,
            val_name,
            x_reg_se,
            x_name,
            immutable_x_name,
            res_reg_se,
            store_to_tmp,
            tmp_name,
            cnt_name,
            loop_name,
            p_name,
            d_name,
            r_name,
            direction,
            diag_status,
            rowblock_size,
            blocks_per_row,
            kernels_iter,
            n_kernels_unrolled,
            nops_before_prekernel,
            move_reg,
            move_base,
            premove_base,
        };

        Ok(sptrsv_generator)
        */
    }
}

pub struct Generator {
    /*
    static_iter: Option<u8>,

    nrow_name: &'static str,

    col_stride: u16,
    col_prefetch_info: Option<(PrefetchType, i16)>,
    col_reg_se: (u8, u8),
    col_name: &'static str,

    val_stride: u16,
    val_prefetch_info: Option<(PrefetchType, i16)>,
    val_reg_se: Option<(u8, u8)>,
    val_name: &'static str,

    x_reg_se: (u8, u8),
    x_name: &'static str,
    immutable_x_name: &'static str,

    res_reg_se: (u8, u8),
    store_to_tmp: bool,
    tmp_name: &'static str,

    cnt_name: &'static str,
    loop_name: &'static str,

    p_name: &'static str,
    d_name: &'static str,
    r_name: &'static str,

    direction: Direction,
    diag_status: DiagonalStatus,
    rowblock_size: u8,
    blocks_per_row: u8,
    kernels_iter: u8,
    n_kernels_unrolled: u8,
    nops_before_prekernel: u8,

    move_reg: bool,
    move_base: bool,
    premove_base: bool,
    */
    matrix_format: SparseMatrixFormat,
    direction: Direction,
    static_iter: Option<u8>,

    nrow_name: &'static str,

    col_premove: i16,
    col_prefetch_info: Option<(PrefetchType, u16)>,
    col_preload_dist: u8,
    col_name: &'static str,

    val_premove: i16,
    val_prefetch_info: Option<(PrefetchType, u16)>,
    val_preload_dist: Option<u8>,
    val_name: &'static str,

    x_preload_dist: u8,
    x_name: &'static str,
    immutable_x_name: &'static str,

    tmp_name: &'static str,

    cnt_name: &'static str,
    loop_name: &'static str,

    p_name: &'static str,
    d_name: &'static str,
    r_name: &'static str,

    rowblock_size: u8,
    n_nops: u8,
    store_to_tmp: bool,
    move_reg: bool,
    move_base: bool,
    skip_preload: bool,
}

impl Generator {
    fn diag_status(&self) -> DiagonalStatus {
        match self.matrix_format {
            SparseMatrixFormat::ELL(ell_info) => ell_info.diag,
        }
    }

    fn blocks_per_row(&self) -> u8 {
        2
    }

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

    fn col_offset(&self) -> u16 {
        size_of::<i32>() as u16 * 8
    }

    fn val_offset(&self) -> u16 {
        size_of::<f64>() as u16 * 8
    }

    fn tmp_offset(&self) -> u16 {
        size_of::<f64>() as u16 * 8
    }

    fn col_need(&self) -> u8 {
        self.blocks_per_row() * (self.col_preload_dist + 1)
    }

    fn val_need(&self) -> u8 {
        self.blocks_per_row() * self.val_preload_dist.map_or(0, |d| d + 1)
    }

    fn x_need(&self) -> u8 {
        self.blocks_per_row() * (self.x_preload_dist + 1)
    }

    fn res_need(&self) -> u8 {
        match self.store_to_tmp {
            true => 1,
            false => self.rowblock_size,
        }
    }

    fn col_se(&self) -> (u8, u8) {
        (0, self.col_need())
    }

    fn res_se(&self) -> (u8, u8) {
        (self.col_se().1, self.col_se().1 + self.res_need())
    }

    fn x_se(&self) -> (u8, u8) {
        (self.res_se().1, self.res_se().1 + self.x_need())
    }

    fn val_se(&self) -> Option<(u8, u8)> {
        match self.val_need() {
            0 => None,
            x => Some((self.x_se().1, self.x_se().1 + x)),
        }
    }

    fn col_ls(&self, idx: u8) -> (u8, u8) {
        let col_se = self.col_se();
        let n = (col_se.1 - col_se.0) / self.blocks_per_row();

        let (si, ei) = match self.move_reg {
            true => (0, n - 1),
            false => (idx % n, (n - 1 + idx) % n),
        };
        (
            col_se.0 + si * self.blocks_per_row(),
            col_se.0 + ei * self.blocks_per_row(),
        )
    }

    fn val_ls(&self, idx: u8) -> Option<(u8, u8)> {
        match self.val_se() {
            None => None,
            Some(val_se) => {
                let n = (val_se.1 - val_se.0) / self.blocks_per_row();

                let (si, ei) = match self.move_reg {
                    true => (0, n - 1),
                    false => (idx % n, (n - 1 + idx) % n),
                };
                Some((
                    val_se.0 + si * self.blocks_per_row(),
                    val_se.0 + ei * self.blocks_per_row(),
                ))
            }
        }
    }

    fn x_ls(&self, idx: u8) -> (u8, u8) {
        let x_se = self.x_se();
        let n = (x_se.1 - x_se.0) / self.blocks_per_row();

        let (si, ei) = match self.move_reg {
            true => (0, n - 1),
            false => (idx % n, (n - 1 + idx) % n),
        };
        (
            x_se.0 + si * self.blocks_per_row(),
            x_se.0 + ei * self.blocks_per_row(),
        )
    }

    fn n_kernels_unrolled(&self) -> u8 {
        match self.move_reg {
            true => 1,
            false => {
                let col_groups = self.col_preload_dist + 1;
                let val_groups = self.val_preload_dist.map_or(0, |d| d + 1);
                let x_groups = self.x_preload_dist + 1;
                let res_groups = match self.store_to_tmp {
                    true => 1,
                    false => self.rowblock_size,
                };

                let n_regs = [col_groups, val_groups, x_groups, res_groups];

                tools::lcm(&n_regs)
            }
        }
    }

    fn kernels_iter(&self) -> u8 {
        self.rowblock_size / self.n_kernels_unrolled()
    }
}

enum StateType {
    Preloading,
    InsertingGap,
    Prekerneling,
    Kerneling,
    Kerneled,
    Postkerneling,
    PremovingXPD,
    Accumulating,
    Ending,
}

const RULEBOOK: &'static [Rule<Generator>] = &[
    Rule {
        condition: Condition::Single {
            id: StateType::Preloading as u32,
        },
        callback: |config: &Generator, _rp: &mut RegisterPool, _states: &Vec<State>| {
            let asm = match config.skip_preload {
                true => Assembly::new(),
                false => {
                    let preload_generator = microkernel::PreloadGenerator::new(
                        config.col_premove,
                        config.col_stride(),
                        config.col_offset(),
                        config.col_preload_dist,
                        config.col_se().0,
                        config.col_name,
                        config.val_premove,
                        config.val_stride(),
                        config.val_offset(),
                        config.val_preload_dist.unwrap_or(0),
                        config.val_se().map(|r| r.0),
                        config.val_name,
                        config.x_preload_dist,
                        config.x_se().0,
                        config.immutable_x_name,
                        config.direction,
                        config.blocks_per_row(),
                    );

                    preload_generator.generate()?.empty_line()
                }
            };
            /*
            };
            let asm = match config.static_iter {
                Some(_) => Assembly::new(),
                None => {
                    let preload_generator = microkernel::PreloadGenerator::new(
                        config.col_premove,
                        config.col_stride(),
                        config.col_offset(),
                        config.col_preload_dist,
                        config.col_se().0,
                        config.col_name,
                        config.val_premove,
                        config.val_stride(),
                        config.val_offset(),
                        config.val_preload_dist.unwrap_or(0),
                        config.val_se().map(|r| r.0),
                        config.val_name,
                        config.x_preload_dist,
                        config.x_se().0,
                        config.immutable_x_name,
                        config.direction,
                        config.blocks_per_row(),
                    );

                    preload_generator.generate()?.empty_line()
                }
            };
            */

            let next_id = StateType::InsertingGap as u32;
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
            id: StateType::InsertingGap as u32,
        },
        callback: |config: &Generator, _rp: &mut RegisterPool, _states: &Vec<State>| {
            let asm = match config.n_nops {
                0 => Assembly::new(),
                n_nops => Assembly::new().nop(n_nops),
            };
            let next_id = StateType::Prekerneling as u32;
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
            id: StateType::Prekerneling as u32,
        },
        callback: |config: &Generator, _rp: &mut RegisterPool, _states: &Vec<State>| {
            let iteration_type = match config.static_iter {
                None => IterationType::DynamicIter {
                    rowblock_size: config.rowblock_size,
                    inner_iter: config.kernels_iter(),
                },
                Some(static_iter) => IterationType::StaticIter { iter: static_iter },
            };

            let prekernel_generator = microkernel::PrekernelGenerator::new(
                iteration_type,
                config.nrow_name,
                config.cnt_name,
                config.loop_name,
            );

            let asm = prekernel_generator.generate()?.empty_line();
            let next_id = StateType::Kerneling as u32;
            let states = (0..config.n_kernels_unrolled())
                .map(|idx| State {
                    id: next_id,
                    idx,
                    reg: 0,
                })
                .collect();

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::Single {
            id: StateType::Kerneling as u32,
        },
        callback: |config: &Generator, _rp: &mut RegisterPool, states: &Vec<State>| {
            let kernel_idx = states[0].idx;

            let kernel_generator = microkernel::KernelGenerator::new(
                true,
                config.col_stride(),
                config.col_offset(),
                config.col_prefetch_info,
                config.col_ls(kernel_idx),
                config.col_name,
                config.val_stride(),
                config.val_offset(),
                config.val_prefetch_info,
                config.val_ls(kernel_idx),
                config.val_name,
                config.x_ls(kernel_idx),
                config.immutable_x_name,
                config.res_se(),
                config.tmp_offset(),
                config.tmp_name,
                kernel_idx,
                config.n_kernels_unrolled(),
                config.direction,
                config.blocks_per_row(),
                config.store_to_tmp,
                config.move_reg,
                config.move_base,
            );

            let asm = kernel_generator.generate()?.empty_line();
            let next_id = StateType::Kerneled as u32;
            let states = Vec::from([State {
                id: next_id,
                idx: kernel_idx,
                reg: 0,
            }]);

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::SameId {
            id: StateType::Kerneled as u32,
            n_states: 2,
            idx_dist: 1,
        },
        callback: |_config: &Generator, _rp: &mut RegisterPool, states: &Vec<State>| {
            let id = StateType::Kerneled as u32;
            let idx = states[1].idx;

            let asm = Assembly::new();
            let states = Vec::from([State { id, idx, reg: 0 }]);

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::Single {
            id: StateType::Kerneled as u32,
        },
        callback: |_config: &Generator, _rp: &mut RegisterPool, _states: &Vec<State>| {
            let asm = Assembly::new();
            let next_id = StateType::Postkerneling as u32;
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
            id: StateType::Postkerneling as u32,
        },
        callback: |config: &Generator, _rp: &mut RegisterPool, _states: &Vec<State>| {
            let move_base = match config.move_base {
                false => Some(config.n_kernels_unrolled()),
                true => None,
            };
            let move_base_tmp = match (config.move_base, config.store_to_tmp) {
                (false, true) => Some(config.n_kernels_unrolled()),
                _ => None,
            };
            let restore_tmp = match config.store_to_tmp {
                true => Some(config.rowblock_size),
                false => None,
            };
            let decrease_cnt = config.kernels_iter() > 1;

            let postkernel_generator = microkernel::PostkernelGenerator::new(
                config.direction,
                move_base,
                config.col_stride(),
                config.col_name,
                config.val_stride(),
                config.val_name,
                move_base_tmp,
                config.tmp_name,
                restore_tmp,
                decrease_cnt,
                config.cnt_name,
                config.loop_name,
            );

            let asm = postkernel_generator.generate()?.empty_line();
            let next_id = StateType::Accumulating as u32;
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
            id: StateType::PremovingXPD as u32,
        },
        callback: |config: &Generator, _rp: &mut RegisterPool, _states: &Vec<State>| {
            let offset = size_of::<f64>() as i16 * config.rowblock_size as i16;

            let asm = Assembly::new()
                .sub_immediate(config.x_name, offset)
                .sub_immediate(config.p_name, offset)
                .sub_immediate(config.d_name, offset);
            let states = Vec::new();

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::Single {
            id: StateType::Accumulating as u32,
        },
        callback: |config: &Generator, rp: &mut RegisterPool, _states: &Vec<State>| {
            let dst_name = config.x_name;

            let general_reg_name = config.cnt_name;

            let load_from_tmp = config.store_to_tmp;

            let tmp_offset = size_of::<f64>() as u16 * 8;

            let action = match config.direction {
                Direction::Forward => accumulate::Action::TrsvForward,
                Direction::Backward => accumulate::Action::TrsvBackward,
            };

            let avail_registers = {
                let mut avail = rp.avail_list().clone();

                let avail_iter = {
                    let bpr = config.blocks_per_row();
                    let col_avail_iter = {
                        let reg_e = config.col_se().1;
                        (reg_e - bpr)..reg_e
                    };
                    let x_avail_iter = {
                        let reg_e = config.x_se().1;
                        (reg_e - bpr)..reg_e
                    };
                    let val_avail_iter = match config.val_se() {
                        None => 0..0,
                        Some(reg_se) => (reg_se.1 - bpr)..reg_se.1,
                    };
                    col_avail_iter.chain(x_avail_iter).chain(val_avail_iter)
                };
                avail_iter.for_each(|i| {
                    avail[i as usize] = true;
                });

                avail
            };

            let accumulate_generator = accumulate::Generator::new(
                dst_name,
                general_reg_name,
                config.res_se(),
                load_from_tmp,
                tmp_offset,
                config.tmp_name,
                config.r_name,
                config.p_name,
                config.d_name,
                action,
                config.rowblock_size,
                avail_registers,
                config.diag_status(),
            );

            let asm = accumulate_generator.generate()?.empty_line();
            let next_id = StateType::Ending as u32;
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
            id: StateType::Ending as u32,
        },
        callback: |config: &Generator, _rp: &mut RegisterPool, _states: &Vec<State>| {
            let dynamic_inner_iter = match config.static_iter {
                None => Some(config.kernels_iter()),
                Some(_) => None,
            };

            let end_generator = end::Generator::new(
                config.direction,
                config.rowblock_size,
                dynamic_inner_iter,
                config.x_name,
                config.p_name,
                config.d_name,
                config.r_name,
                config.nrow_name,
                config.cnt_name,
                config.loop_name,
            );

            let asm = end_generator.generate()?;
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
        let mut arr = [true; 32];

        let occupied_iter = {
            let col_iter = self.col_se().0..self.col_se().1;
            let x_iter = self.x_se().0..self.x_se().1;
            let val_iter = match self.val_se() {
                None => 0..0,
                Some(reg_se) => reg_se.0..reg_se.1,
            };
            let res_iter = self.res_se().0..self.res_se().1;

            col_iter.chain(x_iter).chain(val_iter).chain(res_iter)
        };

        occupied_iter.for_each(|i| {
            arr[i as usize] = false;
        });

        arr
    }

    fn initial_states(&self) -> Vec<State> {
        let mut states = Vec::new();

        states.push(State {
            id: StateType::Preloading as u32,
            idx: 0,
            reg: 0,
        });

        match self.direction {
            Direction::Forward => {}
            Direction::Backward => {
                states.push(State {
                    id: StateType::PremovingXPD as u32,
                    idx: 0,
                    reg: 0,
                });
            }
        }

        states
    }
}
