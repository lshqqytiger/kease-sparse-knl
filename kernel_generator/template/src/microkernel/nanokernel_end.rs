use super::Direction;
use core::*;

impl Generator {
    pub fn new(
        direction: Direction,
        move_base: Option<u8>,
        col_stride: u16,
        col_name: &'static str,
        val_stride: u16,
        val_name: &'static str,
        move_base_tmp: Option<u8>,
        tmp_name: &'static str,
        restore_tmp: Option<u8>,
        decrease_cnt: bool,
        cnt_name: &'static str,
        loop_name: &'static str,
    ) -> Self {
        let move_base_cv = {
            let sign = match direction {
                Direction::Forward => 1,
                Direction::Backward => -1,
            };
            match move_base {
                None => None,
                Some(row) => {
                    let col_offset = sign * row as i16 * col_stride as i16;
                    let val_offset = sign * row as i16 * val_stride as i16;
                    Some((col_offset, val_offset))
                }
            }
        };

        let move_base_tmp = match move_base_tmp {
            None => None,
            Some(row) => Some(row as i16 * size_of::<f64>() as i16 * 8),
        };

        let restore_tmp = match restore_tmp {
            None => None,
            Some(row) => Some(row as i16 * -1 * size_of::<f64>() as i16 * 8),
        };

        Generator {
            move_base_cv,
            col_name,
            val_name,
            move_base_tmp,
            tmp_name,
            decrease_cnt,
            cnt_name,
            loop_name,
            restore_tmp,
        }
    }
}

pub struct Generator {
    move_base_cv: Option<(i16, i16)>,
    col_name: &'static str,
    val_name: &'static str,

    move_base_tmp: Option<i16>,
    tmp_name: &'static str,

    decrease_cnt: bool,
    cnt_name: &'static str,
    loop_name: &'static str,

    restore_tmp: Option<i16>,
}

enum StateType {
    MovingBaseCV,
    MovingBaseTmp,
    DecreasingCnt,
    Jumping,
    RestoringTmp,
}

const RULEBOOK: &'static [Rule<Generator>] = &[
    Rule {
        condition: Condition::Single {
            id: StateType::MovingBaseCV as u32,
        },
        callback: |config: &Generator, _rp: &mut RegisterPool, _states: &Vec<State>| {
            let (col_offset, val_offset) = config.move_base_cv.unwrap();

            let asm = Assembly::new()
                .add_immediate(config.col_name, col_offset)
                .add_immediate(config.val_name, val_offset);
            let states = Vec::new();

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::Single {
            id: StateType::MovingBaseTmp as u32,
        },
        callback: |config: &Generator, _rp: &mut RegisterPool, _states: &Vec<State>| {
            let tmp_offset = config.move_base_tmp.unwrap();

            let asm = Assembly::new().add_immediate(config.tmp_name, tmp_offset);
            let states = Vec::new();

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::Single {
            id: StateType::DecreasingCnt as u32,
        },
        callback: |config: &Generator, _rp: &mut RegisterPool, _states: &Vec<State>| {
            let asm = Assembly::new().sub_immediate(config.cnt_name, 0x1);
            let next_id = StateType::Jumping as u32;
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
            id: StateType::Jumping as u32,
        },
        callback: |config: &Generator, _rp: &mut RegisterPool, _states: &Vec<State>| {
            let asm = Assembly::new().jump_nz(config.loop_name);
            let states = Vec::new();

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::Single {
            id: StateType::RestoringTmp as u32,
        },
        callback: |config: &Generator, _rp: &mut RegisterPool, _states: &Vec<State>| {
            let tmp_offset = config.restore_tmp.unwrap();

            let asm = Assembly::new().sub_immediate(config.tmp_name, -tmp_offset);
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

        if self.move_base_cv.is_some() {
            states.push(State {
                id: StateType::MovingBaseCV as u32,
                idx: 0,
                reg: 0,
            });
        }

        if self.move_base_tmp.is_some() {
            states.push(State {
                id: StateType::MovingBaseTmp as u32,
                idx: 0,
                reg: 0,
            });
        }

        if self.decrease_cnt {
            states.push(State {
                id: StateType::DecreasingCnt as u32,
                idx: 0,
                reg: 0,
            });
        }

        if self.restore_tmp.is_some() {
            states.push(State {
                id: StateType::RestoringTmp as u32,
                idx: 0,
                reg: 0,
            });
        }

        states
    }
}
