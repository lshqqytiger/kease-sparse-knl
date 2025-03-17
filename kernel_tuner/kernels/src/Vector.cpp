#include "Vector.hpp"

#include <numa.h>
#define NUMA_ALLOC(TYPE,LENGTH) (TYPE*)numa_alloc_onnode(sizeof(TYPE)*(LENGTH),1)
#define NUMA_FREE(PTR,TYPE,LENGTH) (numa_free((PTR),sizeof(TYPE)*(LENGTH)))

Vector::Vector(int length) {
    this->length = length;
#ifdef NUMA_ALLOC
    this->values = NUMA_ALLOC(double, length);
#else
    this->values = new double[length];
#endif
}

void Vector::change_to_zero() {
    int length = this->length;
    double* values = this->values;

    for (int i=0; i<length; ++i) {
        values[i] = 0.0;
    }
}

void Vector::change_to_one() {
    int length = this->length;
    double* values = this->values;

    for (int i=0; i<length; ++i) {
        values[i] = 1.0;
    }
}

void Vector::change_to_b(const char* nnzs) {
    int length = this->length;
    double* values = this->values;

    for (int i=0; i<length; ++i) {
        values[i] = 26.0 - (double)(nnzs[i]-1);
    }
}


double Vector::norm(const Vector* other) const {
    int length = this->length;
    double* val = this->values;
    double* o_val = other->values;

    double norm = 0;

    for (int i=0; i<length; ++i) {
        norm += (val[i] - o_val[i]) * (val[i] - o_val[i]);
    }

    return norm;
}
