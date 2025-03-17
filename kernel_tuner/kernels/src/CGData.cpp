#include "CGData.hpp"

CGData::CGData(int nrow) {
    this->r = new Vector(nrow);
    this->z = new Vector(nrow);
    this->p = new Vector(nrow);
    this->Ap = new Vector(nrow);
}
