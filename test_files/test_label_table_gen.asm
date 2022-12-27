init:
    ADDI $g0, $zero, 1
    ADDI $g1, $zero, 1
    LOAD $g5, $g8, $g9, @target

loop:
    ADD $g3, $g0, $g1
    ADD $g0, $zero, $g1
    ADD $g1, $zero, $g2
    ADD $g2, $zero, $g1
    
    CMP $g1, $g5
    BGT $g8, $g9, @end
    JUMP $g8, $g9, @loop

end: HALT

data:
    target: .int 7
    int_long: .long 650000000
    half_float:    .half 5.25
    float:
        .float -3104.76171875
    eszet: .char 'ß'
    list: .section 10 [1, 1, 2, 3, 5, 8, 13, 21, 34, 55]

text:
    text_data: .text 20 "Some characters!"
    