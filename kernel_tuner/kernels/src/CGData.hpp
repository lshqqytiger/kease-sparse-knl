#pragma once

#include "Vector.hpp"

class CGData {
  public:
    Vector* r;
    Vector* z;
    Vector* p;
    Vector* Ap;

    CGData(int nrow);
};
