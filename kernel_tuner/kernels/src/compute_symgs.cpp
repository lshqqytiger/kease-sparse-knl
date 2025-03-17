#define LIBPATH "./builds/libsymgs.so"

#include "compute.hpp"

#include <cstdio>
#include <numa.h>
#include <dlfcn.h>

#define B 32

static double* tmp_storage;
static void* lib_handle;

static int (*symgs_ptr)(int, const int*, const int*, const double*, const double*, double*, double*, double*, const double*, const double*);

int init_symgs() {
    if (tmp_storage == 0) {
        if ((tmp_storage = (double*)numa_alloc_onnode(sizeof(double) * 64, 1)) == 0) {
            return 1;
        }
        if ((lib_handle = dlopen(LIBPATH, RTLD_NOW)) == 0) {
            fprintf(stderr, "Error: dlopen() failed: %s\n", dlerror());
            return 2;
        }
        if ((symgs_ptr = (int (*)(int, const int*, const int*, const double*, const double*, double*, double*, double*, const double*, const double*))dlsym(lib_handle, "symgs")) == 0) {
            return 3;
        }
    }
    return 0;
}

int compute_symgs(const SparseMatrix *A, const Vector *r, Vector *x) {
    const int     nrow = A->nrow;
    const int*    ucol = A->cols + 16;
    const int*    lcol = A->cols;
    const double* uval = A->vals + 16;
    const double* lval = A->vals;
    double*         xv = x->values;
    double*        tmp = tmp_storage;
    double*         pv = A->tmp;
    const double*   dv = A->diag;
    const double*   rv = r->values;

    return symgs_ptr(nrow, ucol, lcol, uval, lval, xv, tmp, pv, dv, rv);
}

int compute_symgs_ref(const SparseMatrix *A, const Vector *r, Vector *x_ref) {
    const int     nrow = A->nrow;
    const int*    ucol = A->cols + 16;
    const int*    lcol = A->cols;
    const double* uval = A->vals + 16;
    const double* lval = A->vals;
    double*         xv = x_ref->values;
    double*        tmp = tmp_storage;
    double*          p = A->tmpr;
    const double*    D = A->diag;
    const double*   rv = r->values;

    // p = -Ux
    {
        for (int i = 0; i < nrow; ++i) {
            double sum = 0.0;

            for (char j = 0; j < B/2; ++j) {
                sum -= uval[j] * xv[ucol[j]];
            }
            
            p[i] = sum;

            ucol += B;
            uval += B;
        }
    }

    // x = trsv(D+L, r+p)
    // p = Dx-p
    {
        for (int i = 0; i < nrow; ++i) {
            double sum = rv[i];

            for (char j = 0; j < B/2; ++j) {
                sum -= lval[j] * xv[lcol[j]];
            }
            xv[i] += (sum + p[i]) / D[i];
            p[i] = sum;
            
            lcol += B;
            lval += B;
        }
    }
    
    // x = trsv(D+U, p)
    {
        for (int i = nrow - 1; i >= 0; --i) {
            ucol -= B;
            uval -= B;

            double sum = p[i];

            for (char j = 0; j < B/2; ++j) {
                sum -= uval[j] * xv[ucol[j]];
            }

            xv[i] = sum / D[i];
        }
    }

    return 0;
}
