namespace MemoryMachine.Tests;

public static class Programs
{
    public static string Program1 = @"# Programa que lee los elementos la cinta de entrada hasta
# que encuentre un 0 y copia los valores leídos a la cinta
# de salida (el valor 0 no se copia)

lee:	read 1
			load 1
			jzero fin
			write 1
			jump lee
fin:	halt
";

    public static string Program2 = @"# Programa que reconoce el lenguaje formado por las
# palabras que tienen el mismo número de 1 y 2.
# Las palabras finalizan con el número 0.
# Los únicos símbolos posibles de entrada son 0, 1 y 2.
# La cinta de salida contendrá un 1 si se reconoce la palabra
# y un 0 si no pertenece al lenguaje

                    LOAD =0
                    STORE 2
                    READ 1
while:		LOAD 1
                    JZERO end_wh
                    LOAD 1
                    SUB =1
                    JZERO else
                    LOAD 2
                    SUB =1
                    STORE 2
                    JUMP end_if
else:			LOAD 2
                    ADD =1
                    STORE 2
end_if:		READ 1
                    JUMP while
end_wh:		LOAD 2
                    JZERO iguales
                    WRITE =0
                    JUMP fin
iguales:	WRITE =1
fin:			HALT";

    public static string Program3 = @"# Programa con una instrucción ilegal (STORE =2)
# Debe producir un error en la ejecución

	LOAD =3
	STORE 1
	ADD =2
	STORE =2
	WRITE 1
	WRITE 2
	HALT
";

    public static string Program4 = @"# Programa que lee los elementos la cinta de entrada hasta
# que encuentre un 0 y escribe el doble del valor leído a la
# cinta de salida (el valor 0 no se copia)

bucle:	READ 1
				LOAD 1
				JZERO fin
				LOAD 1
				MUL =2
				STORE 1
				WRITE 1
				JUMP bucle
fin:		HALT
";

    public static string Program5 = @"# Programa que lee los elementos la cinta de entrada hasta
# que encuentre un 0 y escribe la suma de todos los elementos
# leídos a la cinta de salida

				READ 1
				LOAD =0
				STORE 2
bucle:	LOAD 1
				JZERO fin
				LOAD 2
				ADD 1
				STORE 2
				READ 1
				JUMP bucle
fin:		WRITE 2
				HALT
";

    public static string Program6 = @"# Programa con una instrucción ilegal (WRITE 0)
# Debe producir un error en la ejecución

	LOAD =3
	STORE 1
	ADD =2
	STORE 2
	WRITE 0
	WRITE 1
	WRITE 2
	HALT
";

    public static string Program7 = @"# Programa que lee los elementos de la cinta de entrada hasta
# que encuentra un 0 y va guardando los valores en registros
# sucesivos comenzando por el registro R3. Luego, multiplica
# todos esos valores por 3 y los escribe en la cinta de salida

# Se usa direccionamiento indirecto mediante el registro R2

				load =3
				store 2

lee:		read 1
				load 1
				jzero fin

				store *2
				load 2
				add =1
				store 2

				jump lee

fin:		load =0
				store *2
				load =3
				store 2

carga:	load *2
				jzero fin2

				mul =3
				store *2
				write *2

				load 2
				add =1
				store 2

				jump carga

fin2: 	halt
";

    public static string ArrayToRegisters = @"# Program that reads array size N and N elements
# Stores elements in consecutive registers starting from R2

        read 0      # Read size into R1
        store 1     # R1 will be our size
        load =0
        store 2     # R2 will be our index register
        load 1
loop:   jzero fin   # If size is 0, we're done

        read 0      # Read next element into R0
        store 3[2]  # Store element at address in R2
        write 3[2]  # Write element to output

        load 2      # Increment register index
        add =1
        store 2

        load 1      # Decrement counter
        sub =1
        store 1

        jump loop   # Continue until done

fin:    halt
";

    public static string InsertionSort = @"# Insertion Sort
        read 0      # Read size into R1
        store 1     # R1 will be our size
        load =0
        store 2     # R2 will be our index register
        load 1
start_load:   jzero fin   # If size is 0, we're done

        read 0      # Read next element into R0
        store 3[2]  # Store element in R3

        load 2      # Increment register index
        add =1
        store 2

        load 1      # Decrement counter
        sub =1
        store 1

        jump start_load   # Continue until done
# Now starts the main program

        load =2
        store 2 # R2 will be j
for:    load  2
        sub   1
        add   =1
        jzero end_for
        load  3[2]
        store 4 # R4 will be key
        load  2
        sub  =1
        store 5 # R5 will be i
while:  load  5
        jgtz  skip
second: load  3[5]
        sub   4
        jgtz skip
        jump end_for
skip:   load  3[5]
        store 6 # R6 will be aux
        load  5
        add   =1
        store 7 # R7 will be i+1
        load  6
        store 3[7]
        load  5
        sub   =1
        store 5
        jump  while
end_while:  load  5
        add  =1
        store 6 # R6 will be i+1
        load  4
        store 3[6]
        jump  for
end_for: halt
";

    public static string ModOperator = @"# Program that reads two numbers and calculates the remainder
# of the division of the first number by the second number

        read 0      # Read first number into R0
        store 1     # Store first number in R1
        read 0      # Read second number into R0
        store 2     # Store second number in R2

        load 1
        mod 2
        write 0

        halt
";
}
