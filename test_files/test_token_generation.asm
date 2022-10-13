init: ADDI $g0, $zero, 1
    ADDI $g1, $zero, 0b0001
    LOAD $g5, $g8, $g9, @target

loop: ADD $g3, $g0, $g1
    ADD $g0, $zero, $g1
    ADD $g1, $zero, $g2
    ADD $g2, $zero, $g1
    
    CMP $g1, $g5
    BGT $g8, $g9, @end
    JUMP $g8, $g9, @loop

end: HALT

data:
    target: .int 30000
