// lightweight main.cpp

#include <cstdio>
#include <cstdlib>
#include <cmath>
#include <omp.h>
#include <math.h>

#include "SparseMatrix.hpp"
#include "Vector.hpp"
#include "compute.hpp"

#define SPMV_FLOPS(NROW) ((NROW)*27*2)
#define SPTRSV_FLOPS(NROW) ((NROW)*14*2)
#define SYMGS_FLOPS(NROW) ((NROW)*(26*2+1)*2)

static int parse_arguments(int argc, const char* argv[], int* type, int* n, int* iter);
static int init_problem(SparseMatrix *A, Vector* b, Vector* x, Vector* x_ref, int type);
static int validate(const SparseMatrix *A, const Vector* b, Vector* x, Vector* x_ref, int type);
static int do_flops(const SparseMatrix *A, const Vector* b, Vector* x, int iter, int type);

int main(int argc, const char* argv[]) {
    int type;
    int n;
    int iter;

    if (parse_arguments(argc, argv, &type, &n, &iter) != 0) {
        return 1;
    }

    auto *A = new SparseMatrix(n);
    Vector* b = new Vector(n*n*n);
    Vector* x = new Vector(n*n*n);
    Vector* x_ref = new Vector(n*n*n);

    if (init_problem(A, b, x, x_ref, type) != 0) {
        return 1;
    }
    if (validate(A, b, x, x_ref, type) != 0) {
        return 1;
    }
    if (do_flops(A, b, x, iter, type) != 0) {
        return 1;
    }

    return 0;
}

static int parse_arguments(int argc, const char* argv[], int* type, int* n, int* iter) {
    if (argc != 4) {
        fprintf(stderr, "Error: parse_arguments() failed\n");
        return 1;
    }

    *type = atoi(argv[1]);
    *n    = atoi(argv[2]);
    *iter = atoi(argv[3]);

    if (!(0 <= *type && *type < 6) || !(*n%8 == 0)) {
        fprintf(stderr, "Error: parse_arguments() failed\n");
        return 1;
    }

    return 0;
}

static int init_problem(SparseMatrix *A, Vector* b, Vector* x, Vector* x_ref, int type) {
    int err;
    A->change_to_problem();
    b->change_to_b(A->nnzs);
    x->change_to_one();
    x_ref->change_to_one();

    const int mg_level = 4;
    SparseMatrix *mat = A;
    for (int lv = 1; lv<mg_level; ++lv) {
        mat->generate_coarse_problem();
        mat = mat->Ac;
    }

    if (type/2 == 0) {
        if ((err = init_spmv()) != 0) {
            fprintf(stderr, "Error: init_spmv() failed. err=%d\n", err);
            return 1;
        }
    }
    else if (type/2 == 1) {
        if ((err = init_sptrsv()) != 0) {
            fprintf(stderr, "Error: init_sptrsv() failed. err=%d\n", err);
            return 1;
        }
    }
    else if (type/2 == 2) {
        if ((err = init_symgs()) != 0) {
            fprintf(stderr, "Error: init_symgs() failed. err=%d\n", err);
            return 1;
        }
    }

    return 0;
}

static int validate(const SparseMatrix *A, const Vector* b, Vector* x, Vector* x_ref, int type) {
    if ((type & 1) == 1) {
        return 0;
    }

    double norm = 0;

    if (type/2 == 0) {
        if (compute_spmv(A, b, x) != 0) {
            fprintf(stderr, "Error: compute_spmv() failed\n");
            return 1;
        }
        if (compute_spmv_ref(A, b, x_ref) != 0) {
            fprintf(stderr, "Error: compute_spmv_ref() failed\n");
            return 1;
        }
        norm = x->norm(x_ref);
    }
    else if (type/2 == 1) {
        if (compute_sptrsv(A, b, x) != 0) {
            fprintf(stderr, "Error: compute_sptrsv() failed\n");
            return 1;
        }
        if (compute_sptrsv_ref(A, b, x_ref) != 0) {
            fprintf(stderr, "Error: compute_sptrsv_ref() failed\n");
            return 1;
        }
        norm = x->norm(x_ref);
    }
    else if (type/2 == 2) {
        if (compute_symgs(A, b, x) != 0) {
            fprintf(stderr, "Error: compute_symgs() failed\n");
            return 1;
        }
        if (compute_symgs_ref(A, b, x_ref) != 0) {
            fprintf(stderr, "Error: compute_symgs_ref() failed\n");
            return 1;
        }
        norm = x->norm(x_ref);
    }

    if (norm > 1e-5 || isnan(norm)) {
        fprintf(stderr, "Wrong Answer: norm: %e\n", norm);
        return 1;
    }
    return 0;
}

static int do_flops(const SparseMatrix *A, const Vector* b, Vector* x, int iter, int type) {
    const int n = A->n;

    const double start_time = omp_get_wtime();

    if (type/2 == 0) {
        for (int i=0; i<iter; ++i) {
            compute_spmv(A, b, x);
        }
    }
    else if (type/2 == 1) {
        for (int i=0; i<iter; ++i) {
            compute_sptrsv(A, b, x);
        }
    }
    else if (type/2 == 2) {
        for (int i=0; i<iter; ++i) {
            compute_symgs(A, b, x);
        }
    }

    double end_time = omp_get_wtime();
    double duration = (end_time - start_time) / iter;
    double gflops = 1.0e-9 / duration;
    if (type/2 == 0) {
        gflops *= SPMV_FLOPS(n*n*n);
    }
    else if (type/2 == 1) {
        gflops *= SPTRSV_FLOPS(n*n*n);
    }
    else if (type/2 == 2) {
        gflops *= SYMGS_FLOPS(n*n*n);
    }

    printf("%.5lf sec  %.5lf gflops\n", duration, gflops);

    return 0;
}
