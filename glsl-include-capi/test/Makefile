default: main

main.o: main.c ../include/gli.h
	clang -c main.c -o main.o

main: main.o
	clang main.o -o main -L../../target/debug -lgli

clean:
	rm -f main
	rm -f main.o

run:
	./main
