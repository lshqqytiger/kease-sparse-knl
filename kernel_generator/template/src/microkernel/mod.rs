mod nanokernel;
mod nanokernel_end;
mod nanokernel_start;
mod preload;

pub use nanokernel::Generator as KernelGenerator;
pub use nanokernel_end::Generator as PostkernelGenerator;
pub use nanokernel_start::Generator as PrekernelGenerator;
pub use preload::Generator as PreloadGenerator;

#[derive(Clone, Copy)]
pub enum Direction {
    Forward,
    Backward,
}

#[derive(Clone, Copy)]
pub enum IterationType {
    StaticIter { iter: u8 },
    DynamicIter { rowblock_size: u8, inner_iter: u8 },
}
