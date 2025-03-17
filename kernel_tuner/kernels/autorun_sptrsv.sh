#!/bin/bash

set -e

for col_prefi in T2 #T0 T1 T2 NTA None
do
    for col_prefd in 4608 #2304
    do
        for col_preld in 0 1 2
        do
            for val_prefi in T2 #T0 T1 T2 NTA None
            do
                for val_prefd in 4608 #2304
                do
                    for val_preld in -1 0 1 2
                    do
                        for x_preld in 0 1 2
                        do
                            for rowblock_size in 8 #8
                            do
                                for nops in 0 #1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31
                                do
                                    for store_to_tmp in t f
                                    do
                                        if [ "$store_to_tmp" = "t" ]
                                        then
                                            res_need=1
                                        elif [ "$store_to_tmp" = "f" ]
                                        then
                                            res_need=$rowblock_size
                                        fi
                                        col_need=$((4 * (col_preld + 1)))
                                        val_need=$((4 * (val_preld + 1)))
                                        x_need=$((4 * (x_preld + 1)))

                                        if [ $((res_need + col_need + x_need + val_need)) -gt 32 ]
                                        then
                                            continue
                                        fi


                                        for move_reg in t #t f
                                        do
                                            for move_base in t f
                                            do
                                                echo "./kernel-generator trsv forward -1 $col_prefi $col_prefd $col_preld $val_prefi $val_prefd $val_preld $x_preld $rowblock_size $nops $store_to_tmp $move_reg $move_base"
                                                ./kernel-generator trsv forward -1 $col_prefi $col_prefd $col_preld $val_prefi $val_prefd $val_preld $x_preld $rowblock_size $nops $store_to_tmp $move_reg $move_base > src/sptrsv.cpp

                                                make libsptrsv -s -B
                                                builds/flops 3 40 50
                                                builds/flops 3 40 50
                                                builds/flops 3 40 50
                                            done
                                        done
                                    done
                                done
                            done
                        done
                    done
                done
            done
        done
    done
done
