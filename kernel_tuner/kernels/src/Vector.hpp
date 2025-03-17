#pragma once

class Vector {
  public:
    int length;
    double* values;

    Vector(int length);
    void change_to_zero();
    void change_to_one();
    void change_to_b(const char* nnzs);
    double norm(const Vector* other) const;
};
