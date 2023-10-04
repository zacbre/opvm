# opvm
opvm is a register based virtual machine written in rust.

## Usage
Check main.rs to see how it's used.

## Examples
### Hello World:
```asm
mov rd, _hello_world
call __println

section .data
    _hello_world: "Hello, World!"
```
Output:
```Hello, World!```
### Advanced Hello World:
```asm
mov rd, _hello
mov re, _world
call __concat
mov rd, r0
call __println

section .data
    _hello: "Hello, "
    _world: "World!"
```
Output:
```Hello, World!```

### Push to stack/pop from stack
opvm comes with a stack, and you can push/pop to/from it.
```asm
mov ra, 128
push ra
pop rd
call __println
```
Output:
```
128
```
### Duplicate top of stack:
```asm
mov ra, 128
push ra
dup
pop rb
pop rd
call __println
```
Output:
```
128
```

### Heap
opvm includes a heap that can be allocated to.
```asm
alloc ra, 10
mov ra[0], 'h'
mov ra[1], 'e'
mov ra[2], 'l'
mov ra[3], 'l'
mov ra[4], 'o'
mov ra[5], '!'
mov rd, ra
call __println
free ra
```
Output:
```
hello!
```

### Basic arithmetic
```asm
; add
mov ra, 10
mov rb, 10
add ra, rb
mov rd, ra
call __println

; sub
mov ra, 10
mov rb, 5
sub ra, rb
mov rd, ra
call __println

; mul
mov ra, 10
mov rb, 10
mul ra, rb
mov rd, ra
call __println

; div
mov ra, 10
mov rb, 5
div ra, rb
mov rd, ra
call __println

; rem
mov ra, 10
mov rb, 6
mod ra, rb
mov rd, ra
call __println

```
Output:
```
20
5
100
2
4
```

### Program Flow:
opvm supports labels and conditional jump statements to allow for dynamic program flow.
```asm
_main:
    mov ra, 10
    jmp _end
    mov ra, 5
_end:
    assert ra, 10
```

### Builtins
opvm includes built in functions that can be injected into the VM to be called by the `call` opcode.
Here's some examples of existing builtins:
```asm
; unix timestamp
call __date_now_unix
mov rd, r0
call __println
```
Output (will be different due to the current time):
```
1696444553
```
`__println` and `__print` are also builtins.

### Debugging
These items will help you debug your program.
```asm
call __dbg_heap
call __dbg_print
```

### Error Handling
opvm comes with a built in stack trace generator that will tell you exactly where the code failed.
```asm
mov ra, 10
mov rb, 20
mov rc, 31
add ra, rb
assert ra, rc
```
Output:
```
Error: Assertion failed at 4.
===== Stack Trace =====
0	 | mov ra, 10
1	 | mov rb, 20
2	 | mov rc, 31
3	 | add ra, rb
4	 | assert ra, rc <-- error occurred here
===== App Stack =====
```

### Objects
`todo`

### Example: Random Password Generator
This program will generate a random set of passwords
```asm
section .data
    _chars: "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
    _len: 10
    
section .code
    _main:
        call _generate_password
        add r9, 1                   ; add 1 to r9 (amount of times password generated)
        test r9, 5                  ; compare r9 to 5, set flags
        jl _main                    ; if r9 < 5, jump back to _main
        jmp _stop                   ; unconditionally jmp to _stop
    _clamp_numbers:
        call __random               ; generate a random between 0 and 1
        mov r1, 62                  ; maximum number = 62
        mul r0, r1                  ; multiply random number by 62
        call __floor                 ; round down to nearest integer
        ret
    _generate_password:
        alloc ra, _len              ; allocate _len bytes to ra
        mov rb, _chars              ; copy _chars to rb
        mov rc, 0                   ; copy 0 to rc
        mov r5, _len                ; copy _len to r5
    _loop:
        call _clamp_numbers         ; r0 = return of _clamp_numbers
        mov ra[rc], rb[r0]          ; copy rb[r0] offset (rb is a pointer to _chars
        add rc, 1                   ; add 1 to rc
        test rc, r5                 ; compare rc to r5, set flags
        jl _loop                    ; if rc < r5, jump back to _loop
        mov rd, ra                  ; copy ra to rd
        call __println              ; print rd
        free ra                     ; free memory allocation at ra
        ret
    _stop:                          ; end of program
```

Output (will vary since it is random):
```
1a3LzZbetj
GceTC9mODK
jg0dIDEjAR
56P8TQdVrO
5IbpWd9YDZ
```

### Example: Print out the Alphabet
This example prints out characters 97-123 in the ASCII table as readable characters. 
```asm   
_main:
    mov ra, 26          ; length of the alphabet
    mov rb, 97          ; start position of ASCII table
    mov rc, 0
    alloc rd, 26
_loop:
    mov rd[rc], rb
    add rc, 1
    add rb, 1
    test rc, ra
    jl _loop
_end:
    call __println
    free rd
```
Output:
```
abcdefghijklmnopqrstuvwxyz
```