extern "C" int sptrsv(int nrow, const int* col, const double* val, double* x, double* tmp, double* p, const double* d, const double* r) {
    int i;
    double* imm_x = x;

    asm volatile(
    " vmovdqa (%[COL]), %%ymm0                         \t\n"
    " vmovdqa 0x20(%[COL]), %%ymm1                     \t\n"
    " add $0x80, %[COL]                                \t\n"

    " loop_sptrsv:                                     \t\n"

    " kxnorw %%k0, %%k0, %%k1                          \t\n"
    " kxnorw %%k0, %%k0, %%k2                          \t\n"
    " vmovdqa (%[COL]), %%ymm2                         \t\n"
    " vmovdqa 0x20(%[COL]), %%ymm3                     \t\n"
    " vgatherdpd (%[IMM_X],%%ymm0,8), %%zmm5%{%%k1%}   \t\n"
    " vgatherdpd (%[IMM_X],%%ymm1,8), %%zmm6%{%%k2%}   \t\n"
    " vmulpd (%[VAL]), %%zmm5, %%zmm4                  \t\n"
    " vfnmsub231pd 0x40(%[VAL]), %%zmm6, %%zmm4        \t\n"
    " add $0x80, %[COL]                                \t\n"
    " add $0x100, %[VAL]                               \t\n"
    " vmovapd %%zmm2, %%zmm0                           \t\n"
    " vmovapd %%zmm3, %%zmm1                           \t\n"


    " vextractf64x4 $0x1, %%zmm4, %%ymm2               \t\n"
    " vaddpd %%ymm4, %%ymm2, %%ymm2                    \t\n"
    " vextractf128 $0x1, %%ymm2, %%xmm3                \t\n"
    " vaddpd %%xmm2, %%xmm3, %%xmm3                    \t\n"
    " vhaddpd %%xmm3, %%xmm3, %%xmm3                   \t\n"
    " vaddsd (%[R]), %%xmm3, %%xmm3                    \t\n"
    " vmovapd %%xmm3, %%xmm2                           \t\n"
    " vaddsd (%[P]), %%xmm3, %%xmm3                    \t\n"
    " vdivsd (%[D]), %%xmm3, %%xmm3                    \t\n"
    " vaddsd (%[X]), %%xmm3, %%xmm3                    \t\n"
    " vmovsd %%xmm3, (%[X])                            \t\n"
    " vmovsd %%xmm2, (%[P])                            \t\n"

    " add $0x8, %[X]                                   \t\n"
    " add $0x8, %[P]                                   \t\n"
    " add $0x8, %[D]                                   \t\n"
    " add $0x8, %[R]                                   \t\n"
    " sub $0x1, %[NROW]                                \t\n"
    " jnz loop_sptrsv                                  \t\n"
    : [NROW]"+r"(nrow), [COL]"+r"(col), [VAL]"+r"(val), [X]"+r"(x), [IMM_X]"+r"(imm_x), [TMP]"+r"(tmp), [I]"+r"(i), [P]"+r"(p), [D]"+r"(d), [R]"+r"(r)
    :
    : "zmm0", "zmm1", "zmm2", "zmm3", "zmm4", "zmm5", "zmm6", "k1", "k2"
    );

    return 0;
}

