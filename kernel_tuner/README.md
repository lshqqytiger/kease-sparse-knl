# Preparation

0. Ensure you are in the login shell.

1. Load modules.
  `source set_environments.sh`

2. Copy this directory('kernel_tuner') to the scratch.
  `cp -r ../kernel_tuner /scratch/hpc171a03/kernel_tuner`

3. Go to the scratch kernel_tuner directory.
  `cd /scratch/hpc171a03/kernel_tuner/kernel`

4. Build benchmark.
  `make`


# Benchmarking

0. Ensure you are in the KNL shell and current path is kernel_tuner in scratch.

1. Load modules.
  `source set_environments.sh`

2. Go to the main kernel directory.
  `cd kernels`

3. Generate spmv code.
  `kernel-generator <OPTIONS> > src/spmv.cpp`

4. Build spmv library and run benchmark.
  `./run.sh`


# Autotuning

0. Ensure you are in the KNL shell and benchmarking process works correctly.

1. Load modules.
  `source set_environments.sh`

2. Go to the main kernel directory.
  `cd kernels`

3. Run Benchmarking multiple times.
  `./autorun_spmv.sh`


# Parameters of kernel-generator

## spmv

- `col_pft` : column prefetch type [T0, T1, **T2**, NTA, None]
- `col_pfd` : column prefetch distance (integer > 0, **4096**)
- `col_pld` : column preload distance [0, **1**, 2, ...]
- `val_pft` : value prefetch type [T0, T1, **T2**, NTA, None]
- `val_pfd` : value prefetch distance (integer > 0, **4096**)
- `val_pld` : value preload distance [**-1**, 0, 1, 2, ...] (-1 : fused load-add for value data)
- `x_pld` : xv preload distance [0, 1, **2**, ...]
- `rowblock` : rowblock size [1, 2, 4, **8**]
- `nops` : Number of nops [0, 1, ...]
- `store_to_tmp` : store temporary rowblock result to memory (**f**, t)
- `move_reg` : move data on registers for preloading instead of unrolling (f, **t**)
- `move_base` : move base inside of nanokernel (f, **t**)

## trsv

- `direction` : forward / backward (f, b)
- `static_iter` : additional pre/post trsv that iterates constant time for wavefront (0, 1, 2, ...)
- `col_pft` : column prefetch type [T0, T1, **T2**, NTA, None]
- `col_pfd` : column prefetch distance (integer > 0, **4096**)
- `col_pld` : column preload distance [0, **1**, 2, ...]
- `val_pft` : value prefetch type [T0, T1, **T2**, NTA, None]
- `val_pfd` : value prefetch distance (integer > 0, **4096**)
- `val_pld` : value preload distance [**-1**, 0, 1, 2, ...] (-1 : fused load-add for value data)
- `x_pld` : xv preload distance [0, 1, **2**, ...]
- `rowblock` : rowblock size [1, 2, 4, **8**]
- `nops` : Number of nops [0, 1, ...]
- `store_to_tmp` : store temporary rowblock result to memory (**f**, t)
- `move_reg` : move data on registers for preloading instead of unrolling (f, **t**)
- `move_base` : move base inside of nanokernel (f, **t**)

## symgs

- `static_iter` : additional pre/post trsv that iterates constant time for wavefront (0, 1, 2, ...)
- `col_pft` : column prefetch type [T0, T1, **T2**, NTA, None]
- `col_pfd` : column prefetch distance (integer > 0, **4096**)
- `col_pld` : column preload distance [0, **1**, 2, ...]
- `val_pft` : value prefetch type [T0, T1, **T2**, NTA, None]
- `val_pfd` : value prefetch distance (integer > 0, **4096**)
- `val_pld` : value preload distance [**-1**, 0, 1, 2, ...] (-1 : fused load-add for value data)
- `x_pld` : xv preload distance [0, 1, **2**, ...]
- `spmv_rowblock` : rowblock size for precomputing spmv [1, 2, 4, **8**]
- `trsv_rowblock` : rowblock size for forward/backward trsv [1, 2, 4, **8**]
- `nops_c` : Number of nops for precomputing spmv [0, 1, ...]
- `nops_f0` : Number of nops for preforwarding trsv [0, 1, ...]
- `nops_f1` : Number of nops for forwarding trsv [0, 1, ...]
- `nops_f2` : Number of nops for postforwarding trsv [0, 1, ...]
- `nops_b0` : Number of nops for prebackwarding trsv [0, 1, ...]
- `nops_b1` : Number of nops for backwarding trsv [0, 1, ...]
- `nops_b2` : Number of nops for postbackwarding trsv [0, 1, ...]
- `store_to_tmp` : store temporary rowblock result to memory (**f**, t)
- `move_reg` : move data on registers for preloading instead of unrolling (f, **t**)
- `move_base` : move base inside of nanokernel (f, **t**)
