ADDI $g0, $zero, 10
LOAD $g5, $g6, $g7, @test_1
ADDI $g0, $zero, 10
STORE $g0, $g1, $g2, @test_2
ADDI $g0, $zero, 10
BEQ $g3, @test_3
BGT $g6, @test_4