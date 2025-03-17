extern "C" int symgs(int nrow, const int* ucol, const int* lcol, const double* uval, const double* lval, double* x, double* tmp, double* p, const double* d, const double* r) {
    int i;
    int imm_nrow = nrow;
    double* imm_x = x;
    double* imm_p = p;

    asm volatile(
    // --- precomputing start --- //                
    " kxnorw %%k0, %%k0, %%k1                          \t\n"
    " kxnorw %%k0, %%k0, %%k2                          \t\n"
    " vmovdqa (%[UCOL]), %%ymm0                        \t\n"
    " vmovdqa 0x20(%[UCOL]), %%ymm1                    \t\n"
    " vgatherdpd (%[X],%%ymm0,8), %%zmm12%{%%k1%}      \t\n"
    " vgatherdpd (%[X],%%ymm1,8), %%zmm13%{%%k2%}      \t\n"
    " vmovdqa 0x80(%[UCOL]), %%ymm0                    \t\n"
    " vmovdqa 0xa0(%[UCOL]), %%ymm1                    \t\n"
    " add $0x100, %[UCOL]                              \t\n"

    " sar $0x3, %[NROW]                                \t\n"
    " movl $0x8, %[I]                                  \t\n"
    " loop_c:                                          \t\n"

    " vmovupd %%zmm5, %%zmm4                           \t\n"
    " vmovupd %%zmm6, %%zmm5                           \t\n"
    " vmovupd %%zmm7, %%zmm6                           \t\n"
    " vmovupd %%zmm8, %%zmm7                           \t\n"
    " vmovupd %%zmm9, %%zmm8                           \t\n"
    " vmovupd %%zmm10, %%zmm9                          \t\n"
    " vmovupd %%zmm11, %%zmm10                         \t\n"
    " vmovupd (%[UVAL]), %%zmm16                       \t\n"
    " vmovupd 0x40(%[UVAL]), %%zmm17                   \t\n"
    " kxnorw %%k0, %%k0, %%k1                          \t\n"
    " kxnorw %%k0, %%k0, %%k2                          \t\n"
    " prefetcht2 0x1200(%[UCOL])                       \t\n"
    " vmovdqa (%[UCOL]), %%ymm2                        \t\n"
    " vmovdqa 0x20(%[UCOL]), %%ymm3                    \t\n"
    " vgatherdpd (%[X],%%ymm0,8), %%zmm14%{%%k1%}      \t\n"
    " vgatherdpd (%[X],%%ymm1,8), %%zmm15%{%%k2%}      \t\n"
    " vmulpd %%zmm16, %%zmm12, %%zmm11                 \t\n"
    " vfnmsub231pd %%zmm17, %%zmm13, %%zmm11           \t\n"
    " prefetcht2 0x1200(%[UVAL])                       \t\n"
    " prefetcht2 0x1240(%[UVAL])                       \t\n"
    " add $0x80, %[UCOL]                               \t\n"
    " add $0x100, %[UVAL]                              \t\n"
    " vmovupd %%zmm2, %%zmm0                           \t\n"
    " vmovupd %%zmm3, %%zmm1                           \t\n"
    " vmovupd %%zmm14, %%zmm12                         \t\n"
    " vmovupd %%zmm15, %%zmm13                         \t\n"

    " sub $0x1, %[I]                                   \t\n"
    " jnz loop_c                                       \t\n"

    " movl $0x33, %[I]                                 \t\n"
    " kmovw %[I], %%k1                                 \t\n"
    " knotw %%k1, %%k2                                 \t\n"
    " vinsertf64x4 $0x0, %%ymm4, %%zmm8, %%zmm2        \t\n"
    " valignq $0x4, %%zmm4, %%zmm8, %%zmm8             \t\n"
    " vaddpd %%zmm8, %%zmm2, %%zmm2                    \t\n"
    " vinsertf64x4 $0x0, %%ymm5, %%zmm9, %%zmm3        \t\n"
    " valignq $0x4, %%zmm5, %%zmm9, %%zmm9             \t\n"
    " vaddpd %%zmm9, %%zmm3, %%zmm3                    \t\n"
    " vinsertf64x4 $0x0, %%ymm6, %%zmm10, %%zmm4       \t\n"
    " valignq $0x4, %%zmm6, %%zmm10, %%zmm10           \t\n"
    " vaddpd %%zmm10, %%zmm4, %%zmm4                   \t\n"
    " vinsertf64x4 $0x0, %%ymm7, %%zmm11, %%zmm5       \t\n"
    " valignq $0x4, %%zmm7, %%zmm11, %%zmm11           \t\n"
    " vaddpd %%zmm11, %%zmm5, %%zmm5                   \t\n"
    " vmovupd %%zmm4, %%zmm6                           \t\n"
    " vpermpd $0x4e, %%zmm2, %%zmm4%{%%k1%}            \t\n"
    " vpermpd $0x4e, %%zmm6, %%zmm2%{%%k2%}            \t\n"
    " vaddpd %%zmm4, %%zmm2, %%zmm6                    \t\n"
    " vmovupd %%zmm5, %%zmm2                           \t\n"
    " vpermpd $0x4e, %%zmm3, %%zmm5%{%%k1%}            \t\n"
    " vpermpd $0x4e, %%zmm2, %%zmm3%{%%k2%}            \t\n"
    " vaddpd %%zmm5, %%zmm3, %%zmm2                    \t\n"
    " vshufpd $0xaa, %%zmm2, %%zmm6, %%zmm3            \t\n"
    " vshufpd $0x55, %%zmm2, %%zmm6, %%zmm6            \t\n"
    " vaddpd %%zmm6, %%zmm3, %%zmm3                    \t\n"
    " vmovupd %%zmm3, (%[P])                           \t\n"

    " add $0x40, %[P]                                  \t\n"
    " movl $0x8, %[I]                                  \t\n"
    " sub $0x1, %[NROW]                                \t\n"
    " jnz loop_c                                       \t\n"
    " mov %[IMM_P], %[P]                               \t\n"
    // ---  precomputing end  --- //                

    // --- forwarding start --- //                  
    " mov %[IMM_NROW], %[NROW]                         \t\n"

    " kxnorw %%k0, %%k0, %%k1                          \t\n"
    " kxnorw %%k0, %%k0, %%k2                          \t\n"
    " vmovdqa (%[LCOL]), %%ymm0                        \t\n"
    " vmovdqa 0x20(%[LCOL]), %%ymm1                    \t\n"
    " vgatherdpd (%[IMM_X],%%ymm0,8), %%zmm5%{%%k1%}   \t\n"
    " vgatherdpd (%[IMM_X],%%ymm1,8), %%zmm6%{%%k2%}   \t\n"
    " vmovdqa 0x80(%[LCOL]), %%ymm0                    \t\n"
    " vmovdqa 0xa0(%[LCOL]), %%ymm1                    \t\n"
    " add $0x100, %[LCOL]                              \t\n"

    " loop_f1:                                         \t\n"

    " vmovupd (%[LVAL]), %%zmm9                        \t\n"
    " vmovupd 0x40(%[LVAL]), %%zmm10                   \t\n"
    " kxnorw %%k0, %%k0, %%k1                          \t\n"
    " kxnorw %%k0, %%k0, %%k2                          \t\n"
    " prefetcht2 0x1200(%[LCOL])                       \t\n"
    " vmovdqa (%[LCOL]), %%ymm2                        \t\n"
    " vmovdqa 0x20(%[LCOL]), %%ymm3                    \t\n"
    " vgatherdpd (%[IMM_X],%%ymm0,8), %%zmm7%{%%k1%}   \t\n"
    " vgatherdpd (%[IMM_X],%%ymm1,8), %%zmm8%{%%k2%}   \t\n"
    " vmulpd %%zmm9, %%zmm5, %%zmm4                    \t\n"
    " vfnmsub231pd %%zmm10, %%zmm6, %%zmm4             \t\n"
    " prefetcht2 0x1200(%[LVAL])                       \t\n"
    " prefetcht2 0x1240(%[LVAL])                       \t\n"
    " add $0x80, %[LCOL]                               \t\n"
    " add $0x100, %[LVAL]                              \t\n"
    " vmovupd %%zmm2, %%zmm0                           \t\n"
    " vmovupd %%zmm3, %%zmm1                           \t\n"
    " vmovupd %%zmm7, %%zmm5                           \t\n"
    " vmovupd %%zmm8, %%zmm6                           \t\n"


    " vextractf64x4 $0x1, %%zmm4, %%ymm2               \t\n"
    " vaddpd %%ymm4, %%ymm2, %%ymm2                    \t\n"
    " vextractf128 $0x1, %%ymm2, %%xmm3                \t\n"
    " vaddpd %%xmm2, %%xmm3, %%xmm3                    \t\n"
    " vhaddpd %%xmm3, %%xmm3, %%xmm3                   \t\n"
    " vaddsd (%[R]), %%xmm3, %%xmm3                    \t\n"
    " vmovupd %%xmm3, %%xmm2                           \t\n"
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
    " jnz loop_f1                                      \t\n"

    // ---  forwarding end  --- //                  

    // --- backwarding start --- //                 
    " mov %[IMM_NROW], %[NROW]                         \t\n"

    " sub $0x180, %[UCOL]                              \t\n"
    " kxnorw %%k0, %%k0, %%k1                          \t\n"
    " kxnorw %%k0, %%k0, %%k2                          \t\n"
    " vmovdqa 0xa0(%[UCOL]), %%ymm0                    \t\n"
    " vmovdqa 0x80(%[UCOL]), %%ymm1                    \t\n"
    " vgatherdpd (%[IMM_X],%%ymm0,8), %%zmm5%{%%k1%}   \t\n"
    " vgatherdpd (%[IMM_X],%%ymm1,8), %%zmm6%{%%k2%}   \t\n"
    " vmovdqa 0x20(%[UCOL]), %%ymm0                    \t\n"
    " vmovdqa (%[UCOL]), %%ymm1                        \t\n"

    " loop_b1:                                         \t\n"

    " sub $0x80, %[UCOL]                               \t\n"
    " sub $0x100, %[UVAL]                              \t\n"
    " vmovupd 0x40(%[UVAL]), %%zmm9                    \t\n"
    " vmovupd (%[UVAL]), %%zmm10                       \t\n"
    " kxnorw %%k0, %%k0, %%k1                          \t\n"
    " kxnorw %%k0, %%k0, %%k2                          \t\n"
    " prefetcht2 -0x1200(%[UCOL])                      \t\n"
    " vmovdqa 0x20(%[UCOL]), %%ymm2                    \t\n"
    " vmovdqa (%[UCOL]), %%ymm3                        \t\n"
    " vgatherdpd (%[IMM_X],%%ymm0,8), %%zmm7%{%%k1%}   \t\n"
    " vgatherdpd (%[IMM_X],%%ymm1,8), %%zmm8%{%%k2%}   \t\n"
    " vmulpd %%zmm9, %%zmm5, %%zmm4                    \t\n"
    " vfnmsub231pd %%zmm10, %%zmm6, %%zmm4             \t\n"
    " prefetcht2 -0x1200(%[UVAL])                      \t\n"
    " prefetcht2 -0x1240(%[UVAL])                      \t\n"
    " vmovupd %%zmm2, %%zmm0                           \t\n"
    " vmovupd %%zmm3, %%zmm1                           \t\n"
    " vmovupd %%zmm7, %%zmm5                           \t\n"
    " vmovupd %%zmm8, %%zmm6                           \t\n"


    " sub $0x8, %[X]                                   \t\n"
    " sub $0x8, %[P]                                   \t\n"
    " sub $0x8, %[D]                                   \t\n"
    " vextractf64x4 $0x1, %%zmm4, %%ymm2               \t\n"
    " vaddpd %%ymm4, %%ymm2, %%ymm2                    \t\n"
    " vextractf128 $0x1, %%ymm2, %%xmm3                \t\n"
    " vaddpd %%xmm2, %%xmm3, %%xmm3                    \t\n"
    " vhaddpd %%xmm3, %%xmm3, %%xmm3                   \t\n"
    " vaddsd (%[P]), %%xmm3, %%xmm3                    \t\n"
    " vdivsd (%[D]), %%xmm3, %%xmm3                    \t\n"
    " vmovsd %%xmm3, (%[X])                            \t\n"

    " sub $0x1, %[NROW]                                \t\n"
    " jnz loop_b1                                      \t\n"

    // ---  backwarding end  --- //                 

    : [NROW]"+r"(nrow), [IMM_NROW]"+r"(imm_nrow), [UCOL]"+r"(ucol), [LCOL]"+r"(lcol), [UVAL]"+r"(uval), [LVAL]"+r"(lval), [X]"+r"(x), [IMM_X]"+r"(imm_x), [TMP]"+r"(tmp), [I]"+r"(i), [P]"+r"(p), [IMM_P]"+r"(imm_p), [D]"+r"(d), [R]"+r"(r)
    :
    : "zmm0", "zmm1", "zmm2", "zmm3", "zmm4", "zmm5", "zmm6", "zmm7", "zmm8", "zmm9", "zmm10", "zmm11", "zmm12", "zmm13", "zmm14", "zmm15", "zmm16", "zmm17", "k1", "k2"
    );

    return 0;
}

