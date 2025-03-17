use crate::microkernel::Direction;
use core::*;

impl Generator {
    pub fn new(
        col_premove: i16,
        col_stride: u16,
        col_offset: u16,
        col_preload_dist: u8,
        col_reg_s: u8,
        col_name: &'static str,

        val_premove: i16,
        val_stride: u16,
        val_offset: u16,
        val_preload_dist: u8,
        val_reg_s: Option<u8>,
        val_name: &'static str,

        x_preload_dist: u8,
        x_reg_s: u8,
        x_name: &'static str,

        direction: Direction,
        blocks_per_row: u8,
    ) -> Self {
        Generator {
            col_premove,
            col_stride,
            col_offset,
            col_preload_dist,
            col_reg_s,
            col_name,

            val_premove,
            val_stride,
            val_offset,
            val_preload_dist,
            val_reg_s,
            val_name,

            x_preload_dist,
            x_reg_s,
            x_name,

            direction,
            blocks_per_row,
        }
    }
}

pub struct Generator {
    col_premove: i16,
    col_stride: u16,
    col_offset: u16,
    col_preload_dist: u8,
    col_reg_s: u8,
    col_name: &'static str,

    val_premove: i16,
    val_stride: u16,
    val_offset: u16,
    val_preload_dist: u8,
    val_reg_s: Option<u8>,
    val_name: &'static str,

    x_preload_dist: u8,
    x_reg_s: u8,
    x_name: &'static str,

    direction: Direction,
    blocks_per_row: u8,
}

impl Generator {
    fn col_base(&self, idx: u8) -> i16 {
        let idx = match self.direction {
            Direction::Forward => idx,
            Direction::Backward => self.col_blocks_to_load() - idx - 1,
        };

        let row_idx = idx / self.blocks_per_row;
        let block_idx = idx % self.blocks_per_row;

        self.col_stride as i16 * row_idx as i16 + self.col_offset as i16 * block_idx as i16
    }

    fn val_base(&self, idx: u8) -> i16 {
        let idx = match self.direction {
            Direction::Forward => idx,
            Direction::Backward => self.val_blocks_to_preload() - idx - 1,
        };

        let row_idx = idx / self.blocks_per_row;
        let block_idx = idx % self.blocks_per_row;

        self.val_stride as i16 * row_idx as i16 + self.val_offset as i16 * block_idx as i16
    }

    fn col_blocks_to_load(&self) -> u8 {
        self.col_blocks_to_preload() + self.x_blocks_to_preload()
    }

    fn col_blocks_to_preload(&self) -> u8 {
        self.blocks_per_row * self.col_preload_dist
    }

    fn val_blocks_to_preload(&self) -> u8 {
        self.blocks_per_row * self.val_preload_dist
    }

    fn x_blocks_to_preload(&self) -> u8 {
        self.blocks_per_row * self.x_preload_dist
    }

    fn do_premasking(&self) -> bool {
        self.x_blocks_to_preload() <= 4
    }

    fn col_move_base(&self) -> i16 {
        let sign = match self.direction {
            Direction::Forward => 1,
            Direction::Backward => -1,
        };
        let rows = self.col_preload_dist + self.x_preload_dist;
        sign * rows as i16 * self.col_stride as i16 + self.col_premove
    }

    fn val_move_base(&self) -> i16 {
        let sign = match self.direction {
            Direction::Forward => 1,
            Direction::Backward => -1,
        };
        let rows = self.val_preload_dist;
        sign * rows as i16 * self.val_stride as i16 + self.val_premove
    }
}

enum StateType {
    PremovingBase,
    Premasking,
    LoadingColForX,
    PreloadingCol,
    PreloadingX,
    PreloadingVal,
    PostmovingBase,
}

const RULEBOOK: &'static [Rule<Generator>] = &[
    Rule {
        condition: Condition::Single {
            id: StateType::PremovingBase as u32,
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
            id: StateType::Premasking as u32,
        },
        callback: |_config: &Generator, _rp: &mut RegisterPool, states: &Vec<State>| {
            let idx = states[0].idx;
            let k = idx + 1;

            let asm = Assembly::new().mask_on(k);
            let states = Vec::new();

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::Single {
            id: StateType::LoadingColForX as u32,
        },
        callback: |config: &Generator, rp: &mut RegisterPool, states: &Vec<State>| {
            let idx = states[0].idx;
            let reg = rp.get();

            let base = config.col_base(idx);

            let asm = Assembly::new().load_i32x8(reg, config.col_name, base);
            let next_id = StateType::PreloadingX as u32;
            let states = Vec::from([State {
                id: next_id,
                idx,
                reg,
            }]);

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::Single {
            id: StateType::PreloadingX as u32,
        },
        callback: |config: &Generator, rp: &mut RegisterPool, states: &Vec<State>| {
            let idx = states[0].idx;
            let reg_col = states[0].reg;
            let reg_xv = config.x_reg_s + idx;
            let k = idx % 4 + 1;

            rp.free(reg_col);

            let asm = match config.do_premasking() {
                true => Assembly::new(),
                false => Assembly::new().mask_on(k),
            }
            .gather_f64x8(reg_xv, config.x_name, reg_col, k);
            let states = Vec::new();

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::Single {
            id: StateType::PreloadingCol as u32,
        },
        callback: |config: &Generator, _rp: &mut RegisterPool, states: &Vec<State>| {
            let idx = states[0].idx;
            let reg = config.col_reg_s + idx;

            let base = config.col_base(idx + config.x_blocks_to_preload());

            let asm = Assembly::new().load_i32x8(reg, config.col_name, base);
            let states = Vec::new();

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::Single {
            id: StateType::PreloadingVal as u32,
        },
        callback: |config: &Generator, _rp: &mut RegisterPool, states: &Vec<State>| {
            let idx = states[0].idx;
            let reg = config.val_reg_s.unwrap() + idx;

            let base = config.val_base(idx);

            let asm = Assembly::new().load_f64x8(reg, config.val_name, base);
            let states = Vec::new();

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::Single {
            id: StateType::PostmovingBase as u32,
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
];

impl Generate for Generator {
    fn rulebook(&self) -> &'static [Rule<Self>] {
        RULEBOOK
    }

    fn avail_registers(&self) -> [bool; 32] {
        let mut avail_registers = [true; 32];

        let iter = self.x_reg_s..(self.x_reg_s + self.x_blocks_to_preload());
        for i in iter {
            avail_registers[i as usize] = false;
        }

        avail_registers
    }

    fn initial_states(&self) -> Vec<State> {
        let mut states = Vec::new();

        states.push(State {
            id: StateType::PremovingBase as u32,
            idx: 0,
            reg: 0,
        });

        if self.do_premasking() {
            let mask_to_set = self.x_blocks_to_preload();
            for i in 0..mask_to_set {
                states.push(State {
                    id: StateType::Premasking as u32,
                    idx: i,
                    reg: 0,
                });
            }
        }

        let col_blocks_to_load = self.x_blocks_to_preload();
        for i in 0..col_blocks_to_load {
            states.push(State {
                id: StateType::LoadingColForX as u32,
                idx: i,
                reg: 0,
            });
        }

        for i in 0..self.col_blocks_to_preload() {
            states.push(State {
                id: StateType::PreloadingCol as u32,
                idx: i,
                reg: 0,
            });
        }

        for i in 0..self.val_blocks_to_preload() {
            states.push(State {
                id: StateType::PreloadingVal as u32,
                idx: i,
                reg: 0,
            });
        }

        states.push(State {
            id: StateType::PostmovingBase as u32,
            idx: 0,
            reg: 0,
        });

        states
    }
}
