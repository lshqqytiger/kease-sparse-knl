#[derive(Clone, Copy)]
pub enum SparseMatrixFormat {
    ELL(ELLInfo),
}

// 0~14 : l
// 15   : diag
// 16~31: u

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub struct ELLInfo {
    pub diag: DiagonalStatus,
    pub lu: LUStatus,
    pub ordering: GridPointOrdering,
}

impl ELLInfo {
    pub fn new(diag: DiagonalStatus, lu: LUStatus, ordering: GridPointOrdering) -> Self {
        ELLInfo { diag, lu, ordering }
    }
}

#[derive(Clone, Copy)]
pub enum DiagonalStatus {
    Default,
    Excluded,
    ExcludedReciprocal,
}

#[derive(Clone, Copy)]
pub enum LUStatus {
    Default,
    Excluded,
}

#[derive(Clone, Copy)]
pub enum GridPointOrdering {
    Default,
    WaveFront,
}
