MOVLI $g0, @start
MOVLI $g1, @end

loop_start:
    CMP $g0, $g1
    BEQ $g8, $g9, @end
    ADDI $g0, $g1, 1
    JUMP $g8, $g9, @loop_start

end: HALT


data:
    start: .int 50
    end: .int 100
    bad: .text 20 "hello"

text:
    good: .text 20 "world"
