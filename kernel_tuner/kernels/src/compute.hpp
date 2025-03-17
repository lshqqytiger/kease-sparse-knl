#pragma once

#include "CGData.hpp"
#include "SparseMatrix.hpp"
#include "Vector.hpp"

int init_spmv();
int compute_spmv(const SparseMatrix *A, const Vector *x, Vector *y);
int compute_spmv_ref(const SparseMatrix *A, const Vector *x, Vector *y_ref);

int init_sptrsv();
int compute_sptrsv(const SparseMatrix *A, const Vector *r, Vector *x);
int compute_sptrsv_ref(const SparseMatrix *A, const Vector *r, Vector *x_ref);

int init_symgs();
int compute_symgs(const SparseMatrix *A, const Vector *r, Vector *x);
int compute_symgs_ref(const SparseMatrix *A, const Vector *r, Vector *x_ref);

int compute_cg(const SparseMatrix *A, CGData* data, const Vector* b, Vector* x);

int compute_ddot(const int n, const Vector* x, const Vector* y, double* result);


