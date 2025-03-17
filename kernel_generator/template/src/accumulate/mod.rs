use core::sparse_matrix::*;
use core::*;

pub enum Action {
    Move,
    TrsvForward,
    TrsvBackward,
}

impl Generator {
    pub fn new(
        dst_name: &'static str,

        general_reg_name: &'static str,

        res_reg_se: (u8, u8),
        load_from_tmp: bool,
        tmp_offset: u16,
        tmp_name: &'static str,

        r_name: &'static str,
        p_name: &'static str,
        d_name: &'static str,

        action: Action,
        rowblock_size: u8,
        avail_registers: [bool; 32],
        diag_status: DiagonalStatus,
    ) -> Self {
        let tmp_offset = match load_from_tmp {
            true => Some(tmp_offset as i16),
            false => None,
        };
        let avail_registers_except_res = avail_registers;

        let reversed_res = match action {
            Action::Move | Action::TrsvForward => false,
            Action::TrsvBackward => true,
        };

        let init_mask = match rowblock_size {
            1 => false,
            8 => true,
            _ => panic!("not implemented"),
        };

        Generator {
            dst_name,
            general_reg_name,
            res_reg_se,
            load_from_tmp,
            tmp_offset,
            tmp_name,
            r_name,
            p_name,
            d_name,
            action,
            rowblock_size,
            avail_registers_except_res,
            diag_status,
            reversed_res,
            init_mask,
        }
    }
}

pub struct Generator {
    dst_name: &'static str,

    general_reg_name: &'static str,

    res_reg_se: (u8, u8),
    load_from_tmp: bool,
    tmp_offset: Option<i16>,
    tmp_name: &'static str,

    r_name: &'static str,
    p_name: &'static str,
    d_name: &'static str,

    action: Action,
    rowblock_size: u8,
    avail_registers_except_res: [bool; 32],
    diag_status: DiagonalStatus,
    reversed_res: bool,
    init_mask: bool,
}

enum StateType {
    InitializingMask,
    Loading,

    Lv0F64x8,
    Lv1F64x8,
    Lv2F64x8,
    Lv3F64x8,

    Lv1F64x4,
    Lv2F64x4,
    Lv3F64x4,

    Lv2F64x2,
    Lv3F64x2,

    Lv3F64x1,

    Finalizing,
}

/**
 * TODO:
 * Lv0F64x8 * 4 -> Lv1F64x8 * 2
 * Lv1F64x8 * 2 -> Lv2F64x8 * 1
 * Lv2F64x8 * 1 -> Lv3F64x4 * 1
 * Lv3F64x4 * 1 -> ()
 *
 * Lv0F64x8 * 2 -> Lv1F64x8 * 1
 * Lv1F64x8 * 1 -> Lv2F64x4 * 1
 * Lv2F64x4 * 1 -> Lv3F64x2 * 1
 * Lv3F64x2 * 1 -> ()
 *
 * Lv0F64x8 * 1 -> Lv1F64x4 * 1
 * Lv1F64x4 * 1 -> Lv2F64x2 * 1
 * Lv2F64x2 * 1 -> Lv3F64x1 * 1
 * Lv3F64x1 * 1 -> ()
 */
const RULEBOOK: &'static [Rule<Generator>] = &[
    Rule {
        condition: Condition::Single {
            id: StateType::InitializingMask as u32,
        },
        callback: |config: &Generator, _rp: &mut RegisterPool, _states: &Vec<State>| {
            let asm = Assembly::new().init_mix2mask(config.general_reg_name, 1, 2);
            let states = Vec::new();

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::Single {
            id: StateType::Loading as u32,
        },
        callback: |config: &Generator, rp: &mut RegisterPool, states: &Vec<State>| {
            let idx = states[0].idx;
            let reg = rp.get();

            let base = {
                let i = match config.reversed_res {
                    false => idx,
                    true => config.rowblock_size - idx - 1,
                };
                i as i16 * config.tmp_offset.unwrap()
            };
            let asm = Assembly::new().load_f64x8(reg, config.tmp_name, base);
            let next_id = StateType::Lv0F64x8 as u32;
            let states = Vec::from([State {
                id: next_id,
                idx,
                reg,
            }]);

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::SameId {
            id: StateType::Lv0F64x8 as u32,
            n_states: 2,
            idx_dist: 4,
        },
        callback: |_config: &Generator, rp: &mut RegisterPool, states: &Vec<State>| {
            let idx = states[0].idx;
            let reg0 = states[0].reg;
            let reg1 = states[1].reg;
            let reg2 = rp.get();
            rp.free(reg0);
            rp.free(reg1);

            let asm = Assembly::new().mix4add_f64x8(reg2, reg0, reg1);
            let next_id = StateType::Lv1F64x8 as u32;
            let states = Vec::from([State {
                id: next_id,
                idx,
                reg: reg2,
            }]);

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::SameId {
            id: StateType::Lv0F64x8 as u32,
            n_states: 2,
            idx_dist: 1,
        },
        callback: |_config: &Generator, rp: &mut RegisterPool, states: &Vec<State>| {
            let id = StateType::Lv1F64x8 as u32;
            let idx = states[0].idx;
            let reg0 = states[0].reg;
            let reg1 = states[1].reg;
            let reg2 = rp.get();
            rp.free(reg0);
            rp.free(reg1);

            let asm = Assembly::new().mix4add_f64x8(reg2, reg0, reg1);
            let states = Vec::from([State { id, idx, reg: reg2 }]);

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::SameId {
            id: StateType::Lv0F64x8 as u32,
            n_states: 2,
            idx_dist: 2,
        },
        callback: |_config: &Generator, rp: &mut RegisterPool, states: &Vec<State>| {
            let id = StateType::Lv1F64x8 as u32;
            let idx = states[0].idx;
            let reg0 = states[0].reg;
            let reg1 = states[1].reg;
            let reg2 = rp.get();
            rp.free(reg0);
            rp.free(reg1);

            let asm = Assembly::new().mix4add_f64x8(reg2, reg0, reg1);
            let states = Vec::from([State { id, idx, reg: reg2 }]);

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::Single {
            id: StateType::Lv0F64x8 as u32,
        },
        callback: |_config: &Generator, rp: &mut RegisterPool, states: &Vec<State>| {
            let zmm_src = states[0].reg;
            let ymm_dst = rp.get();
            rp.free(zmm_src);

            let asm = Assembly::new().fold4add_f64x8(ymm_dst, zmm_src);
            let next_id = StateType::Lv1F64x4 as u32;
            let states = Vec::from([State {
                id: next_id,
                idx: 0,
                reg: ymm_dst,
            }]);

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::SameId {
            id: StateType::Lv1F64x8 as u32,
            n_states: 2,
            idx_dist: 2,
        },
        callback: |_config: &Generator, rp: &mut RegisterPool, states: &Vec<State>| {
            let idx = states[0].idx;
            let reg0 = states[0].reg;
            let reg1 = states[1].reg;
            let reg2 = rp.get();
            rp.free(reg0);
            rp.free(reg1);

            let asm = Assembly::new().mix2add_f64x8(reg2, reg0, reg1, 1, 2);
            let next_id = StateType::Lv2F64x8 as u32;
            let states = Vec::from([State {
                id: next_id,
                idx,
                reg: reg2,
            }]);

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::SameId {
            id: StateType::Lv1F64x8 as u32,
            n_states: 2,
            idx_dist: 1,
        },
        callback: |_config: &Generator, rp: &mut RegisterPool, states: &Vec<State>| {
            let id = StateType::Lv2F64x8 as u32;
            let idx = states[0].idx;
            let reg0 = states[0].reg;
            let reg1 = states[1].reg;
            let reg2 = rp.get();
            rp.free(reg0);
            rp.free(reg1);

            let asm = Assembly::new().mix2add_f64x8(reg2, reg0, reg1, 1, 2);
            let states = Vec::from([State { id, idx, reg: reg2 }]);

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::Single {
            id: StateType::Lv1F64x8 as u32,
        },
        callback: |_config: &Generator, _rp: &mut RegisterPool, _states: &Vec<State>| {
            let _next_id = StateType::Lv2F64x4 as u32;

            panic!("not implemented");
        },
    },
    Rule {
        condition: Condition::Single {
            id: StateType::Lv1F64x4 as u32,
        },
        callback: |_config: &Generator, rp: &mut RegisterPool, states: &Vec<State>| {
            let ymm_src = states[0].reg;
            let xmm_dst = rp.get();
            rp.free(ymm_src);

            let asm = Assembly::new().fold2add_f64x4(xmm_dst, ymm_src);
            let next_id = StateType::Lv2F64x2 as u32;
            let states = Vec::from([State {
                id: next_id,
                idx: 0,
                reg: xmm_dst,
            }]);

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::SameId {
            id: StateType::Lv2F64x8 as u32,
            n_states: 2,
            idx_dist: 1,
        },
        callback: |_config: &Generator, rp: &mut RegisterPool, states: &Vec<State>| {
            let idx = states[0].idx;
            let reg0 = states[0].reg;
            let reg1 = states[1].reg;
            let reg2 = rp.get();
            rp.free(reg0);
            rp.free(reg1);

            let asm = Assembly::new().mix1add_f64x8(reg2, reg0, reg1);
            let next_id = StateType::Lv3F64x8 as u32;
            let states = Vec::from([State {
                id: next_id,
                idx,
                reg: reg2,
            }]);

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::Single {
            id: StateType::Lv2F64x8 as u32,
        },
        callback: |_config: &Generator, _rp: &mut RegisterPool, _states: &Vec<State>| {
            let _next_id = StateType::Lv3F64x4 as u32;

            panic!("not implemented");
        },
    },
    Rule {
        condition: Condition::Single {
            id: StateType::Lv2F64x4 as u32,
        },
        callback: |_config: &Generator, _rp: &mut RegisterPool, _states: &Vec<State>| {
            let _next_id = StateType::Lv3F64x2 as u32;

            panic!("not implemented");
        },
    },
    Rule {
        condition: Condition::Single {
            id: StateType::Lv2F64x2 as u32,
        },
        callback: |_config: &Generator, _rp: &mut RegisterPool, states: &Vec<State>| {
            let xmm = states[0].reg;

            let asm = Assembly::new().fold1add_f64x2(xmm, xmm);
            let next_id = StateType::Lv3F64x1 as u32;
            let states = Vec::from([State {
                id: next_id,
                idx: 0,
                reg: xmm,
            }]);

            Ok((asm, states))
        },
    },
    Rule {
        condition: Condition::Single {
            id: StateType::Lv3F64x8 as u32,
        },
        callback: |config: &Generator, rp: &mut RegisterPool, states: &Vec<State>| {
            let zmm_res = states[0].reg;

            let asm = match config.action {
                Action::Move => Assembly::new().store_f64x8(config.dst_name, 0x00, zmm_res),
                Action::TrsvForward => {
                    let zmm_tmp = rp.get();
                    let zmm_d = rp.get();
                    rp.free(zmm_tmp);
                    rp.free(zmm_d);

                    let diag_reciprocal = match config.diag_status {
                        DiagonalStatus::Default => panic!("not implemented"),
                        DiagonalStatus::Excluded => false,
                        DiagonalStatus::ExcludedReciprocal => true,
                    };

                    let asm_load_d = match diag_reciprocal {
                        true => Assembly::new().load_f64x8(zmm_d, config.d_name, 0x0),
                        false => Assembly::new(),
                    };
                    let asm_add = Assembly::new()
                        .loadadd_f64x8(zmm_res, zmm_res, config.r_name, 0x0)
                        .move_f64x8(zmm_tmp, zmm_res)
                        .loadadd_f64x8(zmm_res, zmm_res, config.p_name, 0x0);
                    let asm_diag = match diag_reciprocal {
                        true => {
                            Assembly::new().loadmuladd_f64x8(zmm_res, zmm_d, config.dst_name, 0x0)
                        }
                        false => Assembly::new()
                            .loaddiv_f64x8(zmm_res, zmm_res, config.d_name, 0x0)
                            .loadadd_f64x8(zmm_res, zmm_res, config.dst_name, 0x0),
                    };
                    let asm_store = Assembly::new()
                        .store_f64x8(config.dst_name, 0x0, zmm_res)
                        .store_f64x8(config.p_name, 0x0, zmm_tmp);

                    asm_load_d
                        .append(asm_add)
                        .append(asm_diag)
                        .append(asm_store)
                }
                Action::TrsvBackward => {
                    let diag_reciprocal = match config.diag_status {
                        DiagonalStatus::Default => panic!("not implemented"),
                        DiagonalStatus::Excluded => false,
                        DiagonalStatus::ExcludedReciprocal => true,
                    };

                    let asm_add =
                        Assembly::new().loadadd_f64x8(zmm_res, zmm_res, config.p_name, 0x0);
                    let asm_diag = match diag_reciprocal {
                        true => Assembly::new().loadmul_f64x8(zmm_res, zmm_res, config.d_name, 0x0),
                        false => {
                            Assembly::new().loaddiv_f64x8(zmm_res, zmm_res, config.d_name, 0x0)
                        }
                    };
                    let asm_store = Assembly::new().store_f64x8(config.dst_name, 0x0, zmm_res);

                    asm_add.append(asm_diag).append(asm_store)
                }
            };

            rp.free(zmm_res);

            let next_id = StateType::Finalizing as u32;
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
            id: StateType::Lv3F64x4 as u32,
        },
        callback: |_config: &Generator, _rp: &mut RegisterPool, _states: &Vec<State>| {
            panic!("not implemented");
        },
    },
    Rule {
        condition: Condition::Single {
            id: StateType::Lv3F64x2 as u32,
        },
        callback: |_config: &Generator, _rp: &mut RegisterPool, _states: &Vec<State>| {
            panic!("not implemented");
        },
    },
    Rule {
        condition: Condition::Single {
            id: StateType::Lv3F64x1 as u32,
        },
        callback: |config: &Generator, rp: &mut RegisterPool, states: &Vec<State>| {
            let xmm_res = states[0].reg;

            let asm = match config.action {
                Action::Move => Assembly::new().store_f64x1(config.dst_name, 0x0, xmm_res),
                Action::TrsvForward => {
                    let xmm_tmp = rp.get();
                    let xmm_d = rp.get();
                    rp.free(xmm_tmp);
                    rp.free(xmm_d);

                    let diag_reciprocal = match config.diag_status {
                        DiagonalStatus::Default => panic!("not implemented"),
                        DiagonalStatus::Excluded => false,
                        DiagonalStatus::ExcludedReciprocal => true,
                    };

                    let asm_load_d = match diag_reciprocal {
                        true => Assembly::new().load_f64x1(xmm_d, config.d_name, 0x0),
                        false => Assembly::new(),
                    };
                    let asm_add = Assembly::new()
                        .loadadd_f64x1(xmm_res, xmm_res, config.r_name, 0x0)
                        .move_f64x2(xmm_tmp, xmm_res)
                        .loadadd_f64x1(xmm_res, xmm_res, config.p_name, 0x0);
                    let asm_diag = match diag_reciprocal {
                        true => {
                            Assembly::new().loadmuladd_f64x1(xmm_res, xmm_d, config.dst_name, 0x0)
                        }
                        false => Assembly::new()
                            .loaddiv_f64x1(xmm_res, xmm_res, config.d_name, 0x0)
                            .loadadd_f64x1(xmm_res, xmm_res, config.dst_name, 0x0),
                    };
                    let asm_store = Assembly::new()
                        .store_f64x1(config.dst_name, 0x0, xmm_res)
                        .store_f64x1(config.p_name, 0x0, xmm_tmp);

                    asm_load_d
                        .append(asm_add)
                        .append(asm_diag)
                        .append(asm_store)
                }
                Action::TrsvBackward => {
                    let diag_reciprocal = match config.diag_status {
                        DiagonalStatus::Default => panic!("not implemented"),
                        DiagonalStatus::Excluded => false,
                        DiagonalStatus::ExcludedReciprocal => true,
                    };

                    let asm_add =
                        Assembly::new().loadadd_f64x1(xmm_res, xmm_res, config.p_name, 0x0);
                    let asm_diag = match diag_reciprocal {
                        true => Assembly::new().loadmul_f64x1(xmm_res, xmm_res, config.d_name, 0x0),
                        false => {
                            Assembly::new().loaddiv_f64x1(xmm_res, xmm_res, config.d_name, 0x0)
                        }
                    };
                    let asm_store = Assembly::new().store_f64x1(config.dst_name, 0x0, xmm_res);

                    asm_add.append(asm_diag).append(asm_store)
                }
            };
            rp.free(xmm_res);

            let next_id = StateType::Finalizing as u32;
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
            id: StateType::Finalizing as u32,
        },
        callback: |config: &Generator, rp: &mut RegisterPool, _states: &Vec<State>| {
            for i in config.res_reg_se.0..config.res_reg_se.1 {
                rp.alloc(i);
            }

            let asm = Assembly::new();
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
        self.avail_registers_except_res.clone()
    }

    fn initial_states(&self) -> Vec<State> {
        let mut states = Vec::new();

        if self.init_mask {
            states.push(State {
                id: StateType::InitializingMask as u32,
                idx: 0,
                reg: 0,
            });
        }

        for i in 0..self.rowblock_size {
            let (id, reg) = match (self.load_from_tmp, i < self.rowblock_size - 1) {
                (false, _) => (StateType::Lv0F64x8, self.res_reg_se.0 + i),
                (true, true) => (StateType::Loading, 0),
                (true, false) => (StateType::Lv0F64x8, self.res_reg_se.1 - 1),
            };
            let idx = match self.reversed_res {
                false => i,
                true => self.rowblock_size - i - 1,
            };
            states.push(State {
                id: id as u32,
                idx,
                reg,
            });
        }

        states
    }
}
