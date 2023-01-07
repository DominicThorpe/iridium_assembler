init:
	LOAD $zero, $g8, $g9, @directory
	syscall 8
	HALT

text:
	directory: .text 23 "Novels/Mirrormarch.txt"