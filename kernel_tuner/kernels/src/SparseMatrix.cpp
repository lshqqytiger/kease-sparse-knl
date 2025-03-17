#include "SparseMatrix.hpp"

#include <numa.h>
#define NUMA_ALLOC(TYPE,LENGTH) (TYPE*)numa_alloc_onnode(sizeof(TYPE)*(LENGTH),1)
#define NUMA_FREE(PTR,TYPE,LENGTH) (numa_free((PTR),sizeof(TYPE)*(LENGTH)))
#define B 32

SparseMatrix::SparseMatrix(int n) {
    this->n = n;
    this->nrow = 0;
    this->total_nnz = 0;
    this->nnzs = 0;
    this->cols = 0;
    this->vals = 0;
    this->diag = 0;
    this->tmp  = 0;
    this->tmpr = 0;

    this->Ac = 0;
    this->mgData = 0;
}

void SparseMatrix::change_to_problem() {
    const int padding = 32;

    const int n = this->n;
    const int nrow = n * n * n;

    char*    nnzs = NUMA_ALLOC(char,   nrow              );
    int*     cols = NUMA_ALLOC(int,    nrow * B + padding);
    double*  vals = NUMA_ALLOC(double, nrow * B + padding);
    double*  diag = NUMA_ALLOC(double, nrow     + padding);
    double*   tmp = NUMA_ALLOC(double, nrow     + padding);
    double*  tmpr = NUMA_ALLOC(double, nrow     + padding);

    memset(nnzs, 0, sizeof(char)   * (nrow              ));
    memset(cols, 0, sizeof(int)    * (nrow * B + padding));
    memset(vals, 0, sizeof(double) * (nrow * B + padding));
    memset(diag, 0, sizeof(double) * (nrow     + padding));
    memset( tmp, 0, sizeof(double) * (nrow     + padding));
    memset(tmpr, 0, sizeof(double) * (nrow     + padding));

    int total_nnz = 0;

    for (int row=0; row<nrow; ++row) {
        const int iz = row / (n * n);
        const int iy = (row / n) % n;
        const int ix = row % n;

        char ldnnz = 0;
        char unnz = 0;
        int* current_cols = cols + row * B;
        double* current_vals = vals + row * B;

        for (int t=0; t<27; ++t) {
            const int sz = t / 9 - 1;
            const int sy = (t / 3) % 3 - 1;
            const int sx = t % 3 - 1;

            if (0<=iz+sz && iz+sz<n && 0<=iy+sy && iy+sy<n && 0<=ix+sx && ix+sx<n) {
                const int col = row + sz*n*n + sy*n + sx;

                if (col <= row) {
                    current_vals[ldnnz] = (col == row ? 26.0 : -1.0);
                    current_cols[ldnnz] = col;
                    ldnnz += 1;
                }
                else {
                    current_vals[unnz + B/2] = -1.0;
                    current_cols[unnz + B/2] = col;
                    unnz += 1;
                }
            }
        }

        diag[row] = 26.0;
        nnzs[row] = ldnnz + unnz;
        total_nnz += ldnnz + unnz;
    }

    this->nrow = nrow;
    this->total_nnz = total_nnz;
    this->nnzs = nnzs;
    this->cols = cols;
    this->vals = vals;
    this->diag = diag;
    this->tmp  = tmp;
    this->tmpr = tmpr;
}

void SparseMatrix::generate_coarse_problem() {
    int n = this->n;
    int nc = n / 2;

    int nrowc = nc * nc * nc;
#ifdef NUMA_ALLOC
    int* f2cOperator = NUMA_ALLOC(int, nrowc);
#else
    int* f2cOperator = new int[nrowc];
#endif

    for (int i=0; i<nrowc; ++i) {
        f2cOperator[i] = 0;
    }

    for (int izc=0; izc<nc; ++izc) {
        int iz = 2*izc;
        for (int iyc=0; iyc<nc; ++iyc) {
            int iy = 2*iyc;
            for (int ixc=0; ixc<nc; ++ixc) {
                int ix = 2*ixc;
                
                int rowc = izc*nc*nc + iyc*nc + ixc;
                int row = iz*n*n + iy*n + ix;
                f2cOperator[rowc] = row;
            }
        }
    }

    SparseMatrix* Ac = new SparseMatrix(nc);
    Ac->change_to_problem();

    Vector* rc = new Vector(Ac->nrow);
    Vector* xc = new Vector(Ac->nrow);
    Vector* Axf = new Vector(this->nrow);

    this->Ac = Ac;
    MGData* mgData = new MGData(f2cOperator, rc, xc, Axf);
    this->mgData = mgData;
}
