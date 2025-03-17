#pragma once

#include "Vector.hpp"

class MGData {
  public:
    int pre_step;
    int post_step;
    int* f2cOperator;
    Vector* rc;
    Vector* xc;
    Vector* Axf;

    MGData(int* f2cOperator, Vector* rc, Vector* xc, Vector* Axf);
};
