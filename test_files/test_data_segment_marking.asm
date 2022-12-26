init:
    MOVLI $g0, @ascii_start
    MOVLI $g1, @ascii_end

loop:
    OUT $g0, 0b0000
    ADDI $g0, 1
    CMP $g0, $g1
    BEQ $g8, $g9, @end
    JUMP $g8, $g9, @loop

end: HALT

data:
    ascii_start: .int 0x0041
    ascii_end:   .int 0x007A
    