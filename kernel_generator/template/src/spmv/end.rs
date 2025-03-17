use core::*;

impl Generator {
    pub fn new(
        initial_cnt: Option<u8>,
        cnt_name: &'static str,

        y_offset: i16,
        y_name: &'static str,

        nrow_name: &'static str,

        loop_name: &'static str,
    ) -> Self {
        Generator {
            initial_cnt,
            cnt_name,

            y_offset,
            y_name,

            nrow_name,

            loop_name,
        }
    }
}

pub struct Generator {
    initial_cnt: Option<u8>,
    cnt_name: &'static str,

    y_offset: i16,
    y_name: &'static str,

    nrow_name: &'static str,

    loop_name: &'static str,
}

enum StateType {
    MovingY,
    InitializingCnt,
    DecreasingNrow,
    Jumping,
}

const RULEBOOK: &'static [Rule<Generator>] = &[
    Rule {
        condition: Condition::Single {
            id: StateType::MovingY as u32,
        },
        callback: |config: &Generator, _rp: &mut RegisterPool, _states: &Vec<State>| {
            let asm = Assembly::new().add_immediate(config.y_name, config.y_offset);
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
                Assembly::new().set_immediate(config.cnt_name, config.initial_cnt.unwrap() as i16);
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

        states.push(State {
            id: StateType::MovingY as u32,
            idx: 0,
            reg: 0,
        });
        if self.initial_cnt.is_some() {
            states.push(State {
                id: StateType::InitializingCnt as u32,
                idx: 0,
                reg: 0,
            });
        }

        states.push(State {
            id: StateType::DecreasingNrow as u32,
            idx: 0,
            reg: 0,
        });

        states.push(State {
            id: StateType::Jumping as u32,
            idx: 0,
            reg: 0,
        });

        states
    }
}
