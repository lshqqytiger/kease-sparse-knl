#define LIBPATH "./builds/libsptrsv.so"

#include "compute.hpp"

#include <cstdio>
#include <numa.h>
#include <dlfcn.h>

#define B 32

static double* tmp_storage;
static void* lib_handle;
static int (*sptrsv_ptr)(int, const int*, const double*, double*, double*, double*, const double*, const double*);

int init_sptrsv() {
    if (tmp_storage == 0) {
        if ((tmp_storage = (double*)numa_alloc_onnode(sizeof(double) * 64, 1)) == 0) {
            return 1;
        }
        if ((lib_handle = dlopen(LIBPATH, RTLD_NOW)) == 0) {
            fprintf(stderr, "Error: dlopen() failed: %s\n", dlerror());
            return 2;
        }
        if ((sptrsv_ptr = (int (*)(int, const int*, const double*, double*, double*, double*, const double*, const double*))dlsym(lib_handle, "sptrsv")) == 0) {
            return 3;
        }
    }
    return 0;
}

int compute_sptrsv(const SparseMatrix *A, const Vector *r, Vector *x) {
    const int nrow = A->nrow;
    const int* col = A->cols;
    const double* val = A->vals;
    double* xv = x->values;
    double* tmp = tmp_storage;
    double* p = A->tmp;
    const double* d = A->diag;
    const double* rv = r->values;

    return sptrsv_ptr(nrow, col, val, xv, tmp, p, d, rv);
}

int compute_sptrsv_ref(const SparseMatrix *A, const Vector *r, Vector *x_ref) {
    const int nrow = A->nrow;
    const int* col = A->cols;
    const double* val = A->vals;
    double* xv = x_ref->values;
    double* tmp = tmp_storage;
    double* p = A->tmpr;
    const double* d = A->diag;
    const double* rv = r->values;

    for (int i=0; i<nrow; ++i) {
        double sum = rv[i];

        for (char j=0; j<B/2; ++j) {
            sum -= val[j] * xv[col[j]];
        }
        
        xv[i] += (sum + p[i]) / d[i];
        p[i] = sum;

        col += B;
        val += B;
    }

    return 0;
}
