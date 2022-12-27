start:
    MOVLI $g0, @start_num
    MOVLI $g1, @end_num

loop_start:
    ADDI $g0, $g0, 1
    CMP $g0, $g1
    BEQ $g2, $g3, @end
    JUMP $g2, $g3, @loop_start

end: HALT


data:
    start_num: .int 100
    end_num: .int 300
    big_num: .long 7000000
    name: .text 11 "John Smith"
    pi: .half 3.141
    good_pi: .float 3.14159265359
    jan: .text 8 "January"
    alpha: .char 'a'
