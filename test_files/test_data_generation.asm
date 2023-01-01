start:
    LOAD $g0, $g9, $zero, @start_num
    LOAD $g1, $g9, $zero, @end_num

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
    pi: .half 3.141
    good_pi: .float 3.14159265359
    alpha: .char 'a'

text:
    name: .text 11 "John Smith"
    jan: .text 8 "January"
