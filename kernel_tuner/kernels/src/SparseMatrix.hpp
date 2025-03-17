#pragma once

#include "MGData.hpp"

class SparseMatrix {
  public:
    int n;
    int nrow;
    int total_nnz;
    char* nnzs;
    int* cols;
    double* vals;
    double* diag;
    mutable double* tmp;
    mutable double* tmpr;

/*
    double** diag;

    char* Unnz;
    int* Uc;
    double* Uv;
    char* Lnnz;
    int* Lc;
    double* Lv;
*/

    class SparseMatrix* Ac;
    MGData* mgData;

    SparseMatrix(int n);
    void change_to_problem();
    void generate_coarse_problem();
};
