#define LIBPATH "./builds/libspmv.so"

#include "compute.hpp"

#include <cstdio>
#include <numa.h>
#include <dlfcn.h>

#define B 32

static double* tmp_storage;
static void* lib_handle;
static int (*spmv_ptr)(int, const int*, const double*, const double*, double*, double*);

int init_spmv() {
    if (tmp_storage == 0) {
        if ((tmp_storage = (double*)numa_alloc_onnode(sizeof(double) * 64, 1)) == 0) {
            return 1;
        }
        if ((lib_handle = dlopen(LIBPATH, RTLD_NOW)) == 0) {
            fprintf(stderr, "Error: dlopen() failed: %s\n", dlerror());
            return 2;
        }
        if ((spmv_ptr = (int (*)(int, const int*, const double*, const double*, double*, double*))dlsym(lib_handle, "spmv")) == 0) {
            return 3;
        }
    }
    return 0;
}

int compute_spmv(const SparseMatrix *A, const Vector *x, Vector *y) {
    const int nrow = A->nrow;
    const int* col = A->cols;
    const double* val = A->vals;
    const double* xv = x->values;
    double* tmp = tmp_storage;
    double* yv = y->values;

    return spmv_ptr(nrow, col, val, xv, tmp, yv);
}

int compute_spmv_ref(const SparseMatrix *A, const Vector *x, Vector *y_ref) {
    const int nrow = A->nrow;
    const int* col = A->cols;
    const double* xv = x->values;
    const double* val = A->vals;
    double* yv = y_ref->values;

    for (int i=0; i<nrow; ++i) {
        double sum = 0.0;

        for (char j=0; j<B; ++j) {
            sum += val[j] * xv[col[j]];
        }
        
        yv[i] = sum;
        col += B;
        val += B;
    }

    return 0;
}
