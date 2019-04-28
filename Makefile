.PHONY: clean test

main.bin: *.cpp
	clang++ -Wall -std=c++17 -g -DRUN_TESTS -o $@ *.cpp

test:
	rustc --test --color=always test.rs -o test.rs.bin
	./test.rs.bin

clean:
	rm -v *.bin

arch:
	rm -vf archive.tgz
	tar vczf archive.tgz *.cpp *.h
