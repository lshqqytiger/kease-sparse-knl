#include "MGData.hpp"

MGData::MGData(int* f2cOperator, Vector* rc, Vector* xc, Vector* Axf) {
    this->pre_step = 1;
    this->post_step = 1;
    this->f2cOperator = f2cOperator;
    this->rc = rc;
    this->xc = xc;
    this->Axf = Axf;
}
