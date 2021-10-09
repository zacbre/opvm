# opvm
opvm is a simple stack based virtual machine written in rust.

## Usage
Check main.rs to see how it's used.

## Examples
### Hello World:
```asm
push "Hello, World!"
print
```
Output:
```Hello, World!```
### Advanced Hello World:
```asm
push "Hello"
push ","
concat
push " "
concat
push "World!"
concat
print
```
Output:
```Hello, World!```

### Swap stack items
This will swap the top 2 stack items with each other.
```asm
push 0
push 1
print
print
```
Output:
```
1
0
```
### Duplicate top of stack:
```asm
push 42
dup
print
print
```
Output:
```
42
42
```

### Basic increment/decrement
```asm
push 5
inc
dup
print
dec
print
```
Output:
```
6
5
```

### Loopy:
This program loops 5 times while counting the amount of loops it's done. It also has the ability to detect if the loop is > 1, and it will print a different string for how many loops were done.
```asm
#data
    .multi " loops!"            ; define our string for > 1 loops
    .single " loop!"            ; define our string for a single loop
    .looptimes 5                ; amount of times we are going to loop
#code
    .main
        push 0                  ; push 0 for start of loop counter
        jmp @loop               ; jump to start of the program
    .decideloopy
        dup                     ; duplicate counter
        push 1                  ; push 1 to compare counter
        jg @multi               ; compare counter, if 1, print single
        push @single            ; push our single string for printing
        ret
    .multi
        push @multi             ; push our multi string for printing
        ret
    .loop
        inc                     ; increment loop counter by 1
        dup                     ; duplicate loop counter for printing
        call @decideloopy       ; call into our .decideloopy function
        concat                  ; concat what text we decided on
        print                   ; X [loop|loops]!
        dup                     ; duplicate loop counter for comparing
        push @looptimes         ; push number of loops we expect
        jl @loop                ; less than expected, jump back to .loop to keep looping
```

Output:
```
1 loop!
2 loops!
3 loops!
4 loops!
5 loops!
```

### Print stack size 
```asm
push 0
dup
dup
dup
dup
load $__stack_size
push "stack size is: "
swap
concat
print
```
Output:
```
stack size is: 5
```
### Heap allocation
```asm
push "My value I want on the heap"
alloc $my_heap_var
store $my_heap_var
load $__stack_size
print
load $my_heap_var
load $__stack_size
print
print
```
Output:
```
0
1
My value I want on the heap
```
