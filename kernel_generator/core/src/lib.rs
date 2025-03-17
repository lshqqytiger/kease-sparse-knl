mod assembly;
pub mod sparse_matrix;
pub mod tools;

pub use assembly::{Assembly, PrefetchType};

use std::collections::HashMap;
use std::fmt;

pub struct RegisterPool {
    avail: [bool; 32],
}

impl RegisterPool {
    pub fn new(avail_registers: [bool; 32]) -> Self {
        RegisterPool {
            avail: avail_registers,
        }
    }

    pub fn get(&mut self) -> u8 {
        for i in 0..32 {
            if self.avail[i] == true {
                self.avail[i] = false;
                return i as u8;
            }
        }
        panic!("[RegisterPool] no avail register remain.");
    }

    pub fn alloc(&mut self, i: u8) -> () {
        assert!(i < 32);
        assert!(self.avail[i as usize] == true);
        self.avail[i as usize] = false;
    }

    pub fn free(&mut self, i: u8) -> () {
        assert!(i < 32);
        self.avail[i as usize] = true;
    }

    pub fn avail_list(&self) -> &[bool; 32] {
        &self.avail
    }
}

pub struct State {
    pub id: u32,
    pub idx: u8,
    pub reg: u8,
}

pub struct Rule<T: ?Sized> {
    pub condition: Condition,
    pub callback: fn(
        config: &T,
        &mut RegisterPool,
        &Vec<State>,
    ) -> Result<(Assembly, Vec<State>), GenerateError>,
}

pub enum Condition {
    Single { id: u32 },
    SameId { id: u32, n_states: u8, idx_dist: u8 },
    SameIdx { id0: u32, id1: u32 },
}

struct StateManager {
    state_map: HashMap<u32, [Option<u8>; 32]>,
}

impl StateManager {
    fn new(states: Vec<State>) -> Self {
        let mut state_manager = StateManager {
            state_map: HashMap::new(),
        };
        state_manager.insert_states(states);

        state_manager
    }

    fn insert_states(&mut self, states: Vec<State>) -> () {
        for state in states.into_iter() {
            match self.state_map.get_mut(&state.id) {
                Some(arr) => {
                    assert!(arr[state.idx as usize].is_none());
                    arr[state.idx as usize] = Some(state.reg);
                }
                None => {
                    let mut arr = [None; 32];
                    arr[state.idx as usize] = Some(state.reg);
                    self.state_map.insert(state.id, arr);
                }
            }
        }
    }

    fn take_single_state(&mut self, state_id: u32) -> Option<State> {
        match self.state_map.get_mut(&state_id) {
            Some(arr) => {
                for i in 0..32 {
                    if arr[i].is_some() {
                        let state = State {
                            id: state_id,
                            idx: i as u8,
                            reg: arr[i].take().unwrap(),
                        };

                        return Some(state);
                    }
                }
                None
            }
            None => None,
        }
    }

    fn take_same_id_states(
        &mut self,
        state_id: u32,
        n_states: u8,
        idx_dist: u8,
    ) -> Option<Vec<State>> {
        assert!(n_states > 0);
        assert!(idx_dist > 0);

        match self.state_map.get_mut(&state_id) {
            Some(arr) => {
                for i in 0..32 - (n_states - 1) * idx_dist {
                    let idx_iter = (0..n_states).map(|x| x * idx_dist + i);

                    let all_avail = idx_iter
                        .clone()
                        .map(|idx| arr[idx as usize].is_some())
                        .fold(true, |acc, x| acc & x);

                    if all_avail {
                        let states = idx_iter
                            .map(|idx| {
                                let state = State {
                                    id: state_id,
                                    idx: idx,
                                    reg: arr[idx as usize].take().unwrap(),
                                };
                                state
                            })
                            .collect();
                        return Some(states);
                    }
                }
                None
            }
            None => None,
        }
    }

    fn take_same_idx_states(&mut self, state_id0: u32, state_id1: u32) -> Option<Vec<State>> {
        assert!(state_id0 != state_id1);

        let idx = match (
            self.state_map.get(&state_id0),
            self.state_map.get(&state_id1),
        ) {
            (Some(arr0), Some(arr1)) => Some((arr0, arr1)),
            (_, _) => None,
        }
        .map(|(arr0, arr1)| {
            for i in 0..32 {
                if arr0[i].is_some() && arr1[i].is_some() {
                    return Some(i);
                }
            }
            None
        })
        .unwrap_or(None);

        match idx {
            Some(idx) => {
                let arr = self.state_map.get_mut(&state_id0).unwrap();
                let s0 = State {
                    id: state_id0,
                    idx: idx as u8,
                    reg: arr[idx as usize].take().unwrap(),
                };

                let arr = self.state_map.get_mut(&state_id1).unwrap();
                let s1 = State {
                    id: state_id1,
                    idx: idx as u8,
                    reg: arr[idx as usize].take().unwrap(),
                };

                Some(Vec::from([s0, s1]))
            }
            None => None,
        }
    }

    fn is_empty(&self) -> bool {
        let mut iter = self
            .state_map
            .iter()
            .map(|(_, arr)| arr)
            .flatten()
            .flatten();

        iter.next().is_none()
    }
}

pub trait Generate {
    fn rulebook<'a>(&'a self) -> &'a [Rule<Self>];
    fn avail_registers(&self) -> [bool; 32];
    fn initial_states(&self) -> Vec<State>;

    fn generate(&self) -> Result<Assembly, GenerateError> {
        let rulebook = self.rulebook();
        let mut register_pool = RegisterPool::new(self.avail_registers());
        let states = self.initial_states();
        let mut state_manager = StateManager::new(states);
        let mut asm = Assembly::new();

        loop {
            let mut is_updated = false;
            for rule in rulebook.iter() {
                let states = match rule.condition {
                    Condition::Single { id } => {
                        state_manager.take_single_state(id).map(|s| Vec::from([s]))
                    }
                    Condition::SameId {
                        id,
                        n_states,
                        idx_dist,
                    } => state_manager.take_same_id_states(id, n_states, idx_dist),
                    Condition::SameIdx { id0, id1 } => state_manager.take_same_idx_states(id0, id1),
                };

                if let Some(states) = states {
                    let (res_asm, next_states) =
                        (rule.callback)(self, &mut register_pool, &states)?;
                    state_manager.insert_states(next_states);
                    asm = asm.append(res_asm);

                    is_updated = true;
                    break;
                }
            }

            if is_updated == false {
                break;
            }
        }

        assert!(state_manager.is_empty(), "Not every state is consumed");
        assert!(
            register_pool.avail_list() == &self.avail_registers(),
            "Register pool is changed"
        );

        Ok(asm)
    }
}

#[derive(Debug)]
pub enum GenerateError {
    RegisterOverflow,
    IllegalUnrollFactor,
}

impl fmt::Display for GenerateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RegisterOverflow => write!(f, "vector register overflow"),
            Self::IllegalUnrollFactor => write!(f, "illegal unroll factor"),
        }
    }
}

impl std::error::Error for GenerateError {}
