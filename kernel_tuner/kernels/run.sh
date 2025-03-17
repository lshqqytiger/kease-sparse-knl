#!/bin/bash

set -e

./kernel-generator spmv T2 4608 1 T2 4608 1 1 8 0 f t t > src/spmv.cpp
make libspmv -s -B
builds/flops 0 160 10

#./kernel-generator trsv forward -1 None 4608 1 None 4608 -1 0 1 0 f t t > src/sptrsv.cpp
#make libsptrsv -s -B
#builds/flops 2 40 50

#./kernel-generator symgs -1 T2 4608 1 T2 4608 0 1 8 1 0 0 0 0 0 0 0 f t t > src/symgs.cpp
#make libsymgs -s -B
#builds/flops 4 40 50
