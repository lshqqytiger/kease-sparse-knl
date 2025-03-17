use super::IterationType;
use core::*;

impl Generator {
    pub fn new(
        iteration_type: IterationType,
        nrow_name: &'static str,
        cnt_name: &'static str,
        loop_name: &'static str,
    ) -> Self {
        let (nrow_divisor, initial_cnt) = match iteration_type {
            IterationType::StaticIter { iter } => {
                let nrow_divisor = None;
                let initial_cnt = match iter {
                    1 => None,
                    iter => Some(iter),
                };
                (nrow_divisor, initial_cnt)
            }
            IterationType::DynamicIter {
                rowblock_size,
                inner_iter,
            } => {
                let nrow_divisor = match rowblock_size {
                    1 => None,
                    size => Some(size),
                };
                let initial_cnt = match inner_iter {
                    1 => None,
                    iter => Some(iter),
                };
                (nrow_divisor, initial_cnt)
            }
        };

        Generator {
            nrow_divisor,
            initial_cnt,
            nrow_name,
            cnt_name,
            loop_name,
        }
    }
}

pub struct Generator {
    nrow_divisor: Option<u8>,
    initial_cnt: Option<u8>,
    nrow_name: &'static str,
    cnt_name: &'static str,
    loop_name: &'static str,
}

enum StateType {
    DividingNrow,
    InitializingCnt,
    Labeling,
}

const RULEBOOK: &'static [Rule<Generator>] = &[
    Rule {
        condition: Condition::Single {
            id: StateType::DividingNrow as u32,
        },
        callback: |config: &Generator, _rp: &mut RegisterPool, _states: &Vec<State>| {
            let asm = match config.nrow_divisor.unwrap() {
                x if x.is_power_of_two() => {
                    let dist = x.ilog2() as u8;
                    Assembly::new().shift_right(config.nrow_name, dist)
                }
                _ => panic!("not implemented"),
            };
            let states = Vec::new();

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::Single {
            id: StateType::InitializingCnt as u32,
        },
        callback: |config: &Generator, _rp: &mut RegisterPool, _states: &Vec<State>| {
            let iter = config.initial_cnt.unwrap() as i16;

            let asm = Assembly::new().set_immediate(config.cnt_name, iter);
            let states = Vec::new();

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::Single {
            id: StateType::Labeling as u32,
        },
        callback: |config: &Generator, _rp: &mut RegisterPool, _states: &Vec<State>| {
            let asm = Assembly::new().label(config.loop_name);
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

        if self.nrow_divisor.is_some() {
            let state = State {
                id: StateType::DividingNrow as u32,
                idx: 0,
                reg: 0,
            };
            states.push(state);
        }

        if self.initial_cnt.is_some() {
            let counter_state = State {
                id: StateType::InitializingCnt as u32,
                idx: 0,
                reg: 0,
            };
            states.push(counter_state);
        }

        let label_state = State {
            id: StateType::Labeling as u32,
            idx: 0,
            reg: 0,
        };
        states.push(label_state);

        states
    }
}
