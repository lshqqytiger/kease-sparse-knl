/**
 * @brief
 *
 * matrix input :
 * - nrow
 *   - X
 * - diag
 *   - is_aligned
 *   - store_reciprocal
 *   - offset
 * - v
 *   - X (in symgs)
 * - c
 *   - X (in symgs)
 * - lc & uc
 *   - does_diag_included (only on l)
 *   - offset (same on lc & uc)
 * - lv & uv
 *   - offset (same on lv & uv)
 * - rv & xv & pv
 *   - X
 * - indexing
 *   - index_enum {0:general, 1:spread}
 *
 * matrix constraint :
 * - block size : 32
 * - l & u block size : 16
 * - rv & xv & pv are all aligned and contiguous
 *
 *
 *
 * kernel implementation input :
 * - global
 *   - col_prefetch_distance
 *   - val_prefetch_distance
 *
 * - spmv
 *   - row block size
 *   - save row block on tmp array
 *
 * - trsv
 *   - init & final manual loop count
 *   - row block size
 *   - save row block on tmp array
 *
 *
 */

double* tmp_storage;

int compute_symgs(int nrow, double* diag, int* lc, double* lv, int* uc,
                  double* uv, double* rv, double* xv, double* pv) {
#ifdef SAMPLE_ASDF
    // p = -Ux
    {
        for (int i = 0; i < nrow; ++i) {
            const char unnz = Unnz[i];
            double sum = 0.0;

            for (char j = 0; j < unnz; ++j) {
                sum -= Uv[j] * xv[Uc[j]];
            }
            p[i] = sum;

            Uc += B / 2;
            Uv += B / 2;
        }
    }
    // x = trsv(D+L, r+p)
    // p = Dx-p
    {
        for (int i = 0; i < nrow; ++i) {
            const char lnnz = Lnnz[i];
            double sum = rv[i];

            for (char j = 0; j < lnnz; ++j) {
                sum -= Lv[j] * xv[Lc[j]];
            }

            // xv[i] = (sum + D[i] * xv[i] + p[i]) / D[i];

            // xv[i] = (sum + p[i]) / D[i] + xv[i];

            // xv[i] += (sum + p[i]) / D[i];

            xv[i] = (sum + p[i]) / D[i];
            p[i] = sum;

            Lc += B / 2;
            Lv += B / 2;
        }
    }

    // x = trsv(D+U, p)
    {
        for (int i = nrow - 1; i >= 0; --i) {
            Uc -= B / 2;
            Uv -= B / 2;

            const char unnz = Unnz[i];
            double sum = p[i];

            for (char j = 0; j < unnz; ++j) {
                sum -= Uv[j] * xv[Uc[j]];
            }
            xv[i] = sum / D[i];
        }
    }
#endif

    /*
     * m256i i00, i01, i02, i03
     * m512d b04, b05, b06, b07
     *
     * m512d c00, c08, c09, c10, c11, c12, c13, c14, c15
     * m512d d16, d17, d18, d19
     */

    /*
        const int     nrow = A->nrow;
        const double* D    = A->D;
        const char*   Lnnz = A->Lnnz;
        const int*    Lc   = A->Lc;
        const double* Lv   = A->Lv;
        const char*   Unnz = A->Unnz;
        const int*    Uc   = A->Uc;
        const double* Uv   = A->Uv;
        const double* rv   = r->values;

        double* xv = x->values;
        double* p  = A->tmp;


        const int* current_cols = A->cols;
        const double* current_vals = A->vals;
        */

    double* const origin_diag = diag;
    double* const origin_xv = xv;
    double* const origin_pv = pv;
    double* cxv = xv;

    int i;
    signed char j;

    i = nrow >> 3;
    j = 8;

    // t0 0x05
    // t1 0x06~0b
    // t2 0x06
    asm volatile(

        " prefetchw 0x000(%[TMP]) \t\n"
        " prefetchw 0x040(%[TMP]) \t\n"
        " prefetchw 0x080(%[TMP]) \t\n"
        " prefetchw 0x0c0(%[TMP]) \t\n"
        " nop \t\n"
        " nop \t\n"
        " nop \t\n"
        " nop \t\n"

        " loop0: \t\n"
        " kxnorw %%k0, %%k0, %%k1 \t\n"
        " kxnorw %%k0, %%k0, %%k2 \t\n"

        " prefetcht2 0x0600(%[UC]) \t\n"

        " vmovdqu 0x00(%[UC]), %%ymm0 \t\n"
        " vmovdqu 0x20(%[UC]), %%ymm1 \t\n"

        " vgatherdpd (%[XV],%%ymm0,8), %%zmm2%{%%k1%} \t\n"
        " prefetcht2 0x0600(%[UV]) \t\n"
        " vgatherdpd (%[XV],%%ymm1,8), %%zmm3%{%%k2%} \t\n"

        " vmulpd 0x00(%[UV]), %%zmm2, %%zmm0 \t\n"
        " prefetcht2 0x0640(%[UV]) \t\n"
        " vfnmsub231pd 0x40(%[UV]), %%zmm3, %%zmm0 \t\n"

        " vmovapd %%zmm0, 0x0(%[TMP]) \t\n"
        " add $0x40, %[TMP] \t\n"
        " add $0x40, %[UC] \t\n"
        " add $0x80, %[UV] \t\n"

        " sub $0x1, %[J] \t\n"
        " jnz loop0 \t\n"

        ////////

        " sub $0x200, %[TMP] \t\n"

        " vmovapd 0x000(%[TMP]), %%zmm7 \t\n"
        " vmovapd 0x040(%[TMP]), %%zmm6 \t\n"
        " vmovapd 0x080(%[TMP]), %%zmm5 \t\n"
        " vmovapd 0x0c0(%[TMP]), %%zmm4 \t\n"
        " vmovapd 0x100(%[TMP]), %%zmm3 \t\n"

        " valignq $0x4, %%zmm7, %%zmm3, %%zmm0 \t\n"
        " vinsertf64x4 $0x0, %%ymm7, %%zmm3, %%zmm7 \t\n"
        " vaddpd %%zmm0, %%zmm7, %%zmm0 \t\n"

        " vmovapd 0x140(%[TMP]), %%zmm7 \t\n"
        " vmovapd 0x180(%[TMP]), %%zmm3 \t\n"

        " valignq $0x4, %%zmm6, %%zmm7, %%zmm1 \t\n"
        " vinsertf64x4 $0x0, %%ymm6, %%zmm7, %%zmm6 \t\n"
        " vaddpd %%zmm1, %%zmm6, %%zmm1 \t\n"

        " vmovapd 0x1c0(%[TMP]), %%zmm6 \t\n"

        " valignq $0x4, %%zmm5, %%zmm3, %%zmm2 \t\n"
        " vinsertf64x4 $0x0, %%ymm5, %%zmm3, %%zmm5 \t\n"
        " vaddpd %%zmm2, %%zmm5, %%zmm2 \t\n"

        " valignq $0x4, %%zmm4, %%zmm6, %%zmm3 \t\n"
        " vinsertf64x4 $0x0, %%ymm4, %%zmm6, %%zmm4 \t\n"
        " vaddpd %%zmm3, %%zmm4, %%zmm3 \t\n"

        " movl $0x33, %%eax \t\n"
        " kmovw %%eax, %%k1 \t\n"
        " knotw %%k1, %%k2 \t\n"

        " vmovapd %%zmm3, %%zmm5 \t\n"
        " vpermpd $0x4e, %%zmm1, %%zmm3%{%%k1%} \t\n"
        " vpermpd $0x4e, %%zmm5, %%zmm1%{%%k2%} \t\n"
        " vaddpd %%zmm3, %%zmm1, %%zmm1 \t\n"

        " vmovapd %%zmm2, %%zmm6 \t\n"
        " vpermpd $0x4e, %%zmm0, %%zmm2%{%%k1%} \t\n"
        " vpermpd $0x4e, %%zmm6, %%zmm0%{%%k2%} \t\n"
        " vaddpd %%zmm2, %%zmm0, %%zmm0 \t\n"

        " vmovapd %%zmm1, %%zmm2 \t\n"
        " vshufpd $0xaa, %%zmm1, %%zmm0, %%zmm1 \t\n"
        " vshufpd $0x55, %%zmm2, %%zmm0, %%zmm0 \t\n"
        " vaddpd %%zmm0, %%zmm1, %%zmm0 \t\n"

        " vmovapd %%zmm0, (%[PV]) \t\n"

        " add $0x40, %[PV] \t\n"

        " mov $0x8, %[J] \t\n"
        " sub $0x1, %[I] \t\n"
        " jnz loop0 \t\n"

        : [UV] "+r"(uv), [UC] "+r"(uc), [I] "+r"(i), [J] "+r"(j), [PV] "+r"(pv),
          [TMP] "+r"(tmp_storage)
        : [XV] "r"(xv)
        : "eax", "zmm0", "zmm1", "zmm2", "zmm3", "zmm4", "zmm5", "zmm6", "zmm7",
          "k1", "k2");

    /////////////////////

    pv = origin_pv;

    i = nrow >> 3;
    asm volatile(

        " prefetchw 0x000(%[TMP]) \t\n"
        " prefetchw 0x040(%[TMP]) \t\n"
        " prefetchw 0x080(%[TMP]) \t\n"
        " prefetchw 0x0c0(%[TMP]) \t\n"
        " nop \t\n"
        " nop \t\n"
        " nop \t\n"
        " nop \t\n"

        " loop1: \t\n"
        " kxnorw %%k0, %%k0, %%k1 \t\n"
        " kxnorw %%k0, %%k0, %%k2 \t\n"

        " prefetcht2 0x0600(%[LC]) \t\n"

        " vmovdqu 0x00(%[LC]), %%ymm0 \t\n"
        " vmovdqu 0x20(%[LC]), %%ymm1 \t\n"

        " vgatherdpd (%[CXV],%%ymm0,8), %%zmm2%{%%k1%} \t\n"
        " prefetcht2 0x0600(%[LV]) \t\n"
        " vgatherdpd (%[CXV],%%ymm1,8), %%zmm3%{%%k2%} \t\n"

        " vmulpd 0x00(%[LV]), %%zmm2, %%zmm0 \t\n"
        " prefetcht2 0x0640(%[LV]) \t\n"
        " vfnmsub231pd 0x40(%[LV]), %%zmm3, %%zmm0 \t\n"

        " vmovapd %%zmm0, 0x0(%[TMP]) \t\n"

        " add $0x40, %[TMP] \t\n"
        " add $0x40, %[LC] \t\n"
        " add $0x80, %[LV] \t\n"

        " sub $0x1, %[J] \t\n"
        " jnz loop1 \t\n"

        ////////

        " sub $0x200, %[TMP] \t\n"

        " vmovapd 0x000(%[TMP]), %%zmm7 \t\n"
        " vmovapd 0x040(%[TMP]), %%zmm6 \t\n"
        " vmovapd 0x080(%[TMP]), %%zmm5 \t\n"
        " vmovapd 0x0c0(%[TMP]), %%zmm4 \t\n"
        " vmovapd 0x100(%[TMP]), %%zmm3 \t\n"

        " valignq $0x4, %%zmm7, %%zmm3, %%zmm0 \t\n"
        " vinsertf64x4 $0x0, %%ymm7, %%zmm3, %%zmm7 \t\n"
        " vaddpd %%zmm0, %%zmm7, %%zmm0 \t\n"

        " vmovapd 0x140(%[TMP]), %%zmm7 \t\n"
        " vmovapd 0x180(%[TMP]), %%zmm3 \t\n"

        " valignq $0x4, %%zmm6, %%zmm7, %%zmm1 \t\n"
        " vinsertf64x4 $0x0, %%ymm6, %%zmm7, %%zmm6 \t\n"
        " vaddpd %%zmm1, %%zmm6, %%zmm1 \t\n"

        " vmovapd 0x1c0(%[TMP]), %%zmm6 \t\n"

        " valignq $0x4, %%zmm5, %%zmm3, %%zmm2 \t\n"
        " vinsertf64x4 $0x0, %%ymm5, %%zmm3, %%zmm5 \t\n"
        " vaddpd %%zmm2, %%zmm5, %%zmm2 \t\n"

        " valignq $0x4, %%zmm4, %%zmm6, %%zmm3 \t\n"
        " vinsertf64x4 $0x0, %%ymm4, %%zmm6, %%zmm4 \t\n"
        " vaddpd %%zmm3, %%zmm4, %%zmm3 \t\n"

        " movl $0x33, %%eax \t\n"
        " kmovw %%eax, %%k1 \t\n"
        " knotw %%k1, %%k2 \t\n"

        " vmovapd %%zmm3, %%zmm5 \t\n"
        " vpermpd $0x4e, %%zmm1, %%zmm3%{%%k1%} \t\n"
        " vpermpd $0x4e, %%zmm5, %%zmm1%{%%k2%} \t\n"
        " vaddpd %%zmm3, %%zmm1, %%zmm1 \t\n"

        " vmovapd %%zmm2, %%zmm6 \t\n"
        " vpermpd $0x4e, %%zmm0, %%zmm2%{%%k1%} \t\n"
        " vpermpd $0x4e, %%zmm6, %%zmm0%{%%k2%} \t\n"
        " vaddpd %%zmm2, %%zmm0, %%zmm0 \t\n"

        " vmovapd %%zmm1, %%zmm2 \t\n"
        " vshufpd $0xaa, %%zmm1, %%zmm0, %%zmm1 \t\n"
        " vshufpd $0x55, %%zmm2, %%zmm0, %%zmm0 \t\n"
        " vaddpd %%zmm0, %%zmm1, %%zmm0 \t\n"

        " vaddpd (%[RV]), %%zmm0, %%zmm0 \t\n"
        " vaddpd (%[PV]), %%zmm0, %%zmm1 \t\n"
        " vmovapd %%zmm0, (%[PV]) \t\n"
        " vdivpd (%[D]), %%zmm1, %%zmm1 \t\n"
        " vmovapd %%zmm1, (%[XV]) \t\n"

        " add $0x40, %[RV] \t\n"
        " add $0x40, %[PV] \t\n"
        " add $0x40, %[D] \t\n"
        " add $0x40, %[XV] \t\n"

        " mov $0x8, %[J] \t\n"
        " sub $0x1, %[I] \t\n"
        " jnz loop1 \t\n"

        : [LV] "+r"(lv), [LC] "+r"(lc), [D] "+r"(diag), [I] "+r"(i),
          [J] "+r"(j), [RV] "+r"(rv), [XV] "+r"(xv), [PV] "+r"(pv),
          [TMP] "+r"(tmp_storage), [CXV] "+r"(cxv)
        :
        : "eax", "zmm0", "zmm1", "zmm2", "zmm3", "zmm4", "zmm5", "zmm6", "zmm7",
          "k1", "k2");

    /*
        // x = trsv(D+L, r+p)
        // p = Dx-p
        {
            for (int i=0; i<nrow; ++i) {
                const char lnnz = Lnnz[i];
                double sum = rv[i];

                for (char j=0; j<lnnz; ++j) {
                    sum -= lv[j] * xv[lc[j]];
                }

                xv[i] = (sum + pv[i]) / diag[i];
                pv[i] = sum;

                lv += B/2;
                lc += B/2;
            }
        }
        */

    diag = origin_diag;
    xv = origin_xv;
    pv = origin_pv;

    // x = trsv(D+U, p)
    /*
{
    for (int i = nrow - 1; i >= 0; --i) {
        uv -= B / 2;
        uc -= B / 2;

        const char unnz = Unnz[i];
        double sum = pv[i];

        for (char j = 0; j < unnz; ++j) {
            sum -= uv[j] * xv[uc[j]];
        }
        xv[i] = sum / diag[i];
    }
}
    */

    return 0;
}