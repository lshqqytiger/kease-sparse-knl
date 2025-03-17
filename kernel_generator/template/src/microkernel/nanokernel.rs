use super::Direction;
use core::*;

impl Generator {
    pub fn new(
        negate: bool,

        col_stride: u16,
        col_offset: u16,
        col_prefetch_info: Option<(PrefetchType, u16)>,
        col_reg_ls: (u8, u8),
        col_name: &'static str,

        val_stride: u16,
        val_offset: u16,
        val_prefetch_info: Option<(PrefetchType, u16)>,
        val_reg_ls: Option<(u8, u8)>,
        val_name: &'static str,

        x_reg_ls: (u8, u8),
        x_name: &'static str,

        res_reg_se: (u8, u8),
        tmp_offset: u16,
        tmp_name: &'static str,

        kernel_idx: u8,
        n_kernels_unrolled: u8,
        direction: Direction,
        blocks_per_row: u8,

        store_to_tmp: bool,
        move_reg: bool,
        move_base: bool,
    ) -> Self {
        Generator {
            negate,

            col_stride,
            col_offset,
            col_prefetch_info,
            col_reg_ls,
            col_name,

            val_stride,
            val_offset,
            val_prefetch_info,
            val_reg_ls,
            val_name,

            x_reg_ls,
            x_name,

            res_reg_se,
            tmp_offset,
            tmp_name,

            kernel_idx,
            n_kernels_unrolled,
            direction,
            blocks_per_row,

            store_to_tmp,
            move_reg,
            move_base,
        }
    }
}

pub struct Generator {
    negate: bool,

    col_stride: u16,
    col_offset: u16,
    col_prefetch_info: Option<(PrefetchType, u16)>,
    col_reg_ls: (u8, u8),
    col_name: &'static str,

    val_stride: u16,
    val_offset: u16,
    val_prefetch_info: Option<(PrefetchType, u16)>,
    val_reg_ls: Option<(u8, u8)>,
    val_name: &'static str,

    x_reg_ls: (u8, u8),
    x_name: &'static str,

    res_reg_se: (u8, u8),
    tmp_offset: u16,
    tmp_name: &'static str,

    kernel_idx: u8,
    n_kernels_unrolled: u8,
    direction: Direction,
    blocks_per_row: u8,

    store_to_tmp: bool,
    move_reg: bool,
    move_base: bool,
}

impl Generator {
    fn rb_idx(&self, idx: u8) -> (u8, u8) {
        let global_idx = match self.move_base {
            true => idx,
            false => self.kernel_idx * self.blocks_per_row + idx,
        };

        let real_idx = match self.direction {
            Direction::Forward => global_idx,
            Direction::Backward => self.n_kernels_unrolled * self.blocks_per_row - global_idx - 1,
        };

        let row_idx = real_idx / self.blocks_per_row;
        let block_idx = real_idx % self.blocks_per_row;

        (row_idx, block_idx)
    }

    fn col_prefetch(&self, idx: u8) -> Option<(PrefetchType, i16)> {
        match self.col_prefetch_info {
            None => None,
            Some((pt, dist)) => {
                let sign = match self.direction {
                    Direction::Forward => 1,
                    Direction::Backward => -1,
                };

                let dist = dist as i16 + idx as i16 * self.col_offset as i16;
                Some((pt, dist * sign))
            }
        }
    }

    fn col_move_base(&self) -> i16 {
        match (self.move_base, self.direction) {
            (false, _) => 0,
            (true, Direction::Forward) => self.col_stride as i16,
            (true, Direction::Backward) => self.col_stride as i16 * -1,
        }
    }

    fn val_prefetch(&self, idx: u8) -> Option<(PrefetchType, i16)> {
        match self.val_prefetch_info {
            None => None,
            Some((pt, dist)) => {
                let sign = match self.direction {
                    Direction::Forward => 1,
                    Direction::Backward => -1,
                };

                let dist = dist as i16 + idx as i16 * self.val_offset as i16;
                Some((pt, dist * sign))
            }
        }
    }

    fn col_reg_to_load(&self, idx: u8) -> u8 {
        self.col_reg_ls.0 + idx
    }

    fn val_reg_to_load(&self, idx: u8) -> Option<u8> {
        match self.val_reg_ls {
            None => None,
            Some(reg_ls) => Some(reg_ls.0 + idx),
        }
    }

    fn x_reg_to_load(&self, idx: u8) -> u8 {
        self.x_reg_ls.0 + idx
    }

    fn col_reg_to_store(&self, idx: u8) -> u8 {
        self.col_reg_ls.1 + idx
    }

    fn val_reg_to_store(&self, idx: u8) -> Option<u8> {
        match self.val_reg_ls {
            None => None,
            Some(reg_ls) => Some(reg_ls.1 + idx),
        }
    }

    fn x_reg_to_store(&self, idx: u8) -> u8 {
        self.x_reg_ls.1 + idx
    }

    fn res_reg(&self) -> u8 {
        match (self.store_to_tmp, self.move_reg) {
            (true, _) => self.res_reg_se.0,
            (false, true) => self.res_reg_se.1 - 1,
            (false, false) => self.res_reg_se.0 + self.kernel_idx,
        }
    }

    fn col_base(&self, idx: u8) -> i16 {
        let (row_idx, block_idx) = self.rb_idx(idx);

        let row_base = row_idx as i16 * self.col_stride as i16;
        let block_base = block_idx as i16 * self.col_offset as i16;

        row_base + block_base
    }

    fn val_base(&self, idx: u8) -> i16 {
        let (row_idx, block_idx) = self.rb_idx(idx);

        let row_base = row_idx as i16 * self.val_stride as i16;
        let block_base = block_idx as i16 * self.val_offset as i16;

        row_base + block_base
    }

    fn val_move_base(&self) -> i16 {
        match (self.move_base, self.direction) {
            (false, _) => 0,
            (true, Direction::Forward) => self.val_stride as i16,
            (true, Direction::Backward) => self.val_stride as i16 * -1,
        }
    }

    fn col_move_reg(&self) -> Vec<(u8, u8)> {
        match self.move_reg {
            false => Vec::new(),
            true => (self.col_reg_ls.0..self.col_reg_ls.1)
                .map(|reg| {
                    let from = reg + self.blocks_per_row;
                    let to = reg;
                    (to, from)
                })
                .collect(),
        }
    }

    fn val_move_reg(&self) -> Vec<(u8, u8)> {
        match (self.move_reg, self.val_reg_ls) {
            (false, _) | (_, None) => Vec::new(),
            (true, Some(reg_ls)) => (reg_ls.0..reg_ls.1)
                .map(|reg| {
                    let from = reg + self.blocks_per_row;
                    let to = reg;
                    (to, from)
                })
                .collect(),
        }
    }

    fn x_move_reg(&self) -> Vec<(u8, u8)> {
        match self.move_reg {
            false => Vec::new(),
            true => (self.x_reg_ls.0..self.x_reg_ls.1)
                .map(|reg| {
                    let from = reg + self.blocks_per_row;
                    let to = reg;
                    (to, from)
                })
                .collect(),
        }
    }

    fn res_move_reg(&self) -> Vec<(u8, u8)> {
        match (self.store_to_tmp, self.move_reg) {
            (true, _) | (_, false) => Vec::new(),
            (false, true) => (self.res_reg_se.0..(self.res_reg_se.1 - 1))
                .map(|reg| {
                    let from = reg + 1;
                    let to = reg;
                    (to, from)
                })
                .collect(),
        }
    }

    fn tmp_base(&self) -> Option<i16> {
        match (self.store_to_tmp, self.move_base) {
            (false, _) => None,
            (true, true) => Some(0),
            (true, false) => Some(self.kernel_idx as i16 * self.tmp_offset as i16),
        }
    }

    fn tmp_move_base(&self) -> Option<i16> {
        match (self.store_to_tmp, self.move_base) {
            (false, _) | (_, false) => None,
            (true, true) => Some(self.tmp_offset as i16),
        }
    }

    fn multiplication_type(&self, idx: u8) -> MultiplicationType {
        if idx == 0 {
            MultiplicationType::Mul
        } else if idx < self.blocks_per_row - 1 || self.negate == false {
            MultiplicationType::MulAdd
        } else {
            MultiplicationType::NMulSub
        }
    }
}

enum MultiplicationType {
    Mul,
    MulAdd,
    NMulSub,
}

enum StateType {
    PremovingBaseCV,
    MovingRes,
    LoadingVal,
    ValLoaded,
    LoadingCol,
    LoadingX,
    XLoaded,
    Multiplying,
    InitializingMask,
    MaskSet,
    Multiplied,
    StoringRes,
    PostmovingBaseCV,
    MovingBaseTmp,
    MovingCVX,
    PrefetchingCol,
    PrefetchingVal,
}

const RULEBOOK: &'static [Rule<Generator>] = &[
    Rule {
        condition: Condition::Single {
            id: StateType::PremovingBaseCV as u32,
        },
        callback: |config: &Generator, _rp: &mut RegisterPool, _states: &Vec<State>| {
            let asm = {
                let asm_movcol = match config.col_move_base() {
                    base if base < 0 => Assembly::new().sub_immediate(config.col_name, -base),
                    _ => Assembly::new(),
                };
                let asm_movval = match config.val_move_base() {
                    base if base < 0 => Assembly::new().sub_immediate(config.val_name, -base),
                    _ => Assembly::new(),
                };

                asm_movcol.append(asm_movval)
            };
            let states = Vec::new();

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::Single {
            id: StateType::MovingRes as u32,
        },
        callback: |config: &Generator, _rp: &mut RegisterPool, _states: &Vec<State>| {
            let asm = config
                .res_move_reg()
                .into_iter()
                .map(|(to, from)| Assembly::new().move_f64x8(to, from))
                .fold(Assembly::new(), |acc, x| acc.append(x));
            let states = Vec::new();

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::Single {
            id: StateType::LoadingVal as u32,
        },
        callback: |config: &Generator, _rp: &mut RegisterPool, states: &Vec<State>| {
            let idx = states[0].idx;

            let asm = match config.val_reg_to_store(idx) {
                None => Assembly::new(),
                Some(reg) => {
                    let base = config.val_base(idx);
                    Assembly::new().load_f64x8(reg, config.val_name, base)
                }
            };
            let next_id = StateType::ValLoaded as u32;
            let states = Vec::from([State {
                id: next_id,
                idx,
                reg: 0,
            }]);

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::Single {
            id: StateType::InitializingMask as u32,
        },
        callback: |_config: &Generator, _rp: &mut RegisterPool, states: &Vec<State>| {
            let idx = states[0].idx;
            let k = idx + 1;

            let asm = Assembly::new().mask_on(k);
            let next_id = StateType::MaskSet as u32;
            let states = Vec::from([State {
                id: next_id,
                idx,
                reg: 0,
            }]);

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::Single {
            id: StateType::PrefetchingCol as u32,
        },
        callback: |config: &Generator, _rp: &mut RegisterPool, states: &Vec<State>| {
            let idx = states[0].idx;

            let asm = match config.col_prefetch(idx) {
                None => Assembly::new(),
                Some((pt, dist)) => Assembly::new().prefetch(pt, config.col_name, dist),
            };
            let states = Vec::new();

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::Single {
            id: StateType::LoadingCol as u32,
        },
        callback: |config: &Generator, _rp: &mut RegisterPool, states: &Vec<State>| {
            let idx = states[0].idx;

            let asm = {
                let reg = config.col_reg_to_store(idx);
                let base = config.col_base(idx);

                Assembly::new().load_i32x8(reg, config.col_name, base)
            };
            let next_id = StateType::LoadingX as u32;
            let states = Vec::from([State {
                id: next_id,
                idx,
                reg: 0,
            }]);

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::SameIdx {
            id0: StateType::MaskSet as u32,
            id1: StateType::LoadingX as u32,
        },
        callback: |config: &Generator, _rp: &mut RegisterPool, states: &Vec<State>| {
            let idx = states[0].idx;
            let col_reg = config.col_reg_to_load(idx);
            let x_reg = config.x_reg_to_store(idx);
            let k = idx + 1;

            let asm = Assembly::new().gather_f64x8(x_reg, config.x_name, col_reg, k);
            let next_id = StateType::XLoaded as u32;
            let states = Vec::from([State {
                id: next_id,
                idx,
                reg: 0,
            }]);

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::SameIdx {
            id0: StateType::ValLoaded as u32,
            id1: StateType::XLoaded as u32,
        },
        callback: |_config: &Generator, _rp: &mut RegisterPool, states: &Vec<State>| {
            let idx = states[0].idx;

            let asm = Assembly::new();
            let next_id = StateType::Multiplying as u32;
            let states = Vec::from([State {
                id: next_id,
                idx,
                reg: 0,
            }]);

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::Single {
            id: StateType::XLoaded as u32,
        },
        callback: |_config: &Generator, _rp: &mut RegisterPool, states: &Vec<State>| {
            let idx = states[0].idx;

            let asm = Assembly::new();
            let next_id = StateType::Multiplying as u32;
            let states = Vec::from([State {
                id: next_id,
                idx,
                reg: 0,
            }]);

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::Single {
            id: StateType::Multiplying as u32,
        },
        callback: |config: &Generator, _rp: &mut RegisterPool, states: &Vec<State>| {
            let idx = states[0].idx;
            let val_reg = config.val_reg_to_load(idx);
            let x_reg = config.x_reg_to_load(idx);
            let res_reg = config.res_reg();
            let base = config.val_base(idx);

            let asm = match (val_reg, config.multiplication_type(idx)) {
                (Some(val_reg), MultiplicationType::Mul) => {
                    Assembly::new().mul_f64x8(res_reg, x_reg, val_reg)
                }
                (Some(val_reg), MultiplicationType::MulAdd) => {
                    Assembly::new().muladd_f64x8(res_reg, x_reg, val_reg)
                }
                (Some(val_reg), MultiplicationType::NMulSub) => {
                    Assembly::new().nmulsub_f64x8(res_reg, x_reg, val_reg)
                }
                (None, MultiplicationType::Mul) => {
                    Assembly::new().loadmul_f64x8(res_reg, x_reg, config.val_name, base)
                }
                (None, MultiplicationType::MulAdd) => {
                    Assembly::new().loadmuladd_f64x8(res_reg, x_reg, config.val_name, base)
                }
                (None, MultiplicationType::NMulSub) => {
                    Assembly::new().loadnmulsub_f64x8(res_reg, x_reg, config.val_name, base)
                }
            };

            let next_id = StateType::Multiplied as u32;
            let states = Vec::from([State {
                id: next_id,
                idx,
                reg: 0,
            }]);

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::Single {
            id: StateType::PrefetchingVal as u32,
        },
        callback: |config: &Generator, _rp: &mut RegisterPool, states: &Vec<State>| {
            let idx = states[0].idx;

            let asm = match config.val_prefetch(idx) {
                None => Assembly::new(),
                Some((pt, dist)) => Assembly::new().prefetch(pt, config.val_name, dist),
            };
            let states = Vec::new();

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::SameId {
            id: StateType::Multiplied as u32,
            n_states: 2,
            idx_dist: 1,
        },
        callback: |_config: &Generator, _rp: &mut RegisterPool, states: &Vec<State>| {
            let id = StateType::Multiplied as u32;
            let idx = states[1].idx;

            let asm = Assembly::new();
            let states = Vec::from([State { id, idx, reg: 0 }]);

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::Single {
            id: StateType::Multiplied as u32,
        },
        callback: |_config: &Generator, _rp: &mut RegisterPool, _states: &Vec<State>| {
            let asm = Assembly::new();
            let states = Vec::new();

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::Single {
            id: StateType::StoringRes as u32,
        },
        callback: |config: &Generator, _rp: &mut RegisterPool, _states: &Vec<State>| {
            let asm = match config.tmp_base() {
                None => Assembly::new(),
                Some(base) => Assembly::new().store_f64x8(config.tmp_name, base, config.res_reg()),
            };
            let states = Vec::new();

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::Single {
            id: StateType::PostmovingBaseCV as u32,
        },
        callback: |config: &Generator, _rp: &mut RegisterPool, _states: &Vec<State>| {
            let asm = {
                let asm_movcol = match config.col_move_base() {
                    base if base > 0 => Assembly::new().add_immediate(config.col_name, base),
                    _ => Assembly::new(),
                };
                let asm_movval = match config.val_move_base() {
                    base if base > 0 => Assembly::new().add_immediate(config.val_name, base),
                    _ => Assembly::new(),
                };

                asm_movcol.append(asm_movval)
            };
            let states = Vec::new();

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::Single {
            id: StateType::MovingBaseTmp as u32,
        },
        callback: |config: &Generator, _rp: &mut RegisterPool, _states: &Vec<State>| {
            let asm = match config.tmp_move_base() {
                None => Assembly::new(),
                Some(base) => Assembly::new().add_immediate(config.tmp_name, base),
            };
            let states = Vec::new();

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::Single {
            id: StateType::MovingCVX as u32,
        },
        callback: |config: &Generator, _rp: &mut RegisterPool, _states: &Vec<State>| {
            let asm = {
                let asm_col = config
                    .col_move_reg()
                    .into_iter()
                    .map(|(to, from)| Assembly::new().move_f64x8(to, from))
                    .fold(Assembly::new(), |acc, x| acc.append(x));

                let asm_val = config
                    .val_move_reg()
                    .into_iter()
                    .map(|(to, from)| Assembly::new().move_f64x8(to, from))
                    .fold(Assembly::new(), |acc, x| acc.append(x));

                let asm_x = config
                    .x_move_reg()
                    .into_iter()
                    .map(|(to, from)| Assembly::new().move_f64x8(to, from))
                    .fold(Assembly::new(), |acc, x| acc.append(x));

                asm_col.append(asm_val).append(asm_x)
            };
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

        states.push(State {
            id: StateType::PremovingBaseCV as u32,
            idx: 0,
            reg: 0,
        });

        states.push(State {
            id: StateType::MovingRes as u32,
            idx: 0,
            reg: 0,
        });

        if self.val_reg_ls.is_some() {
            for i in 0..self.blocks_per_row {
                states.push(State {
                    id: StateType::LoadingVal as u32,
                    idx: i,
                    reg: 0,
                });
            }
        }

        for i in 0..self.blocks_per_row {
            states.push(State {
                id: StateType::InitializingMask as u32,
                idx: i,
                reg: 0,
            });
        }

        for i in 0..self.blocks_per_row {
            states.push(State {
                id: StateType::LoadingCol as u32,
                idx: i,
                reg: 0,
            });
        }

        if self.col_prefetch_info.is_some() {
            for i in (0..self.blocks_per_row).filter(|x| (x & 1) == 0) {
                states.push(State {
                    id: StateType::PrefetchingCol as u32,
                    idx: i,
                    reg: 0,
                });
            }
        }

        if self.val_prefetch_info.is_some() {
            for i in 0..self.blocks_per_row {
                states.push(State {
                    id: StateType::PrefetchingVal as u32,
                    idx: i,
                    reg: 0,
                });
            }
        }

        if self.store_to_tmp {
            states.push(State {
                id: StateType::StoringRes as u32,
                idx: 0,
                reg: 0,
            });
        }

        states.push(State {
            id: StateType::PostmovingBaseCV as u32,
            idx: 0,
            reg: 0,
        });

        states.push(State {
            id: StateType::MovingBaseTmp as u32,
            idx: 0,
            reg: 0,
        });

        states.push(State {
            id: StateType::MovingCVX as u32,
            idx: 0,
            reg: 0,
        });

        states
    }
}
