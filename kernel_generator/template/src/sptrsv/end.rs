use crate::*;
use core::*;

impl Generator {
    pub fn new(
        direction: Direction,
        rowblock_size: u8,
        dynamic_inner_iter: Option<u8>,

        x_name: &'static str,
        p_name: &'static str,
        d_name: &'static str,
        r_name: &'static str,

        nrow_name: &'static str,
        cnt_name: &'static str,
        loop_name: &'static str,
    ) -> Self {
        let xpd_offset = match direction {
            Direction::Forward => Some(size_of::<f64>() as i16 * rowblock_size as i16),
            Direction::Backward => None,
        };

        let r_offset = match direction {
            Direction::Forward => Some(size_of::<f64>() as i16 * rowblock_size as i16),
            Direction::Backward => None,
        };

        let (init_cnt, decrease_nrow) = match dynamic_inner_iter {
            Some(iter) if iter > 1 => (Some(iter), true),
            Some(_) => (None, true),
            None => (None, false),
        };

        Generator {
            xpd_offset,
            r_offset,
            x_name,
            p_name,
            d_name,
            r_name,
            init_cnt,
            decrease_nrow,
            nrow_name,
            cnt_name,
            loop_name,
        }
    }
}

pub struct Generator {
    xpd_offset: Option<i16>,
    r_offset: Option<i16>,

    x_name: &'static str,
    p_name: &'static str,
    d_name: &'static str,
    r_name: &'static str,

    init_cnt: Option<u8>,
    decrease_nrow: bool,
    nrow_name: &'static str,
    cnt_name: &'static str,
    loop_name: &'static str,
}

enum StateType {
    MovingXPD,
    MovingR,
    InitializingCnt,
    DecreasingNrow,
    DecreasingCnt,
    Jumping,
}

const RULEBOOK: &'static [Rule<Generator>] = &[
    Rule {
        condition: Condition::Single {
            id: StateType::MovingXPD as u32,
        },
        callback: |config: &Generator, _rp: &mut RegisterPool, _states: &Vec<State>| {
            let offset = config.xpd_offset.unwrap();
            let asm = Assembly::new()
                .add_immediate(config.x_name, offset)
                .add_immediate(config.p_name, offset)
                .add_immediate(config.d_name, offset);

            let states = Vec::new();

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::Single {
            id: StateType::MovingR as u32,
        },
        callback: |config: &Generator, _rp: &mut RegisterPool, _states: &Vec<State>| {
            let offset = config.r_offset.unwrap();
            let asm = Assembly::new().add_immediate(config.r_name, offset);

            let states = Vec::new();

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::Single {
            id: StateType::InitializingCnt as u32,
        },
        callback: |config: &Generator, _rp: &mut RegisterPool, _states: &Vec<State>| {
            let asm =
                Assembly::new().set_immediate(config.cnt_name, config.init_cnt.unwrap() as i16);
            let states = Vec::new();

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::Single {
            id: StateType::DecreasingNrow as u32,
        },
        callback: |config: &Generator, _rp: &mut RegisterPool, _states: &Vec<State>| {
            let asm = Assembly::new().sub_immediate(config.nrow_name, 0x1);
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
            let states = Vec::new();

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

        if self.xpd_offset.is_some() {
            states.push(State {
                id: StateType::MovingXPD as u32,
                idx: 0,
                reg: 0,
            });
        }

        if self.r_offset.is_some() {
            states.push(State {
                id: StateType::MovingR as u32,
                idx: 0,
                reg: 0,
            });
        }

        if self.init_cnt.is_some() {
            states.push(State {
                id: StateType::InitializingCnt as u32,
                idx: 0,
                reg: 0,
            });
        }

        {
            let id = match self.decrease_nrow {
                true => StateType::DecreasingNrow,
                false => StateType::DecreasingCnt,
            };
            states.push(State {
                id: id as u32,
                idx: 0,
                reg: 0,
            });
        }

        states.push(State {
            id: StateType::Jumping as u32,
            idx: 0,
            reg: 0,
        });

        states
    }
}
