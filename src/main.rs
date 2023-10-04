#![feature(layout_for_ptr)]

use crate::lexer::lexer::Lexer;
use crate::vm::vm::Vm;

mod lexer;
mod types;
mod vm;

fn main() {
    run(r#"
        mov rd,1
        call __println
    "#
    .to_string());
}

fn run(code: String) -> Vm {
    let lexer = Lexer::new();
    let program = lexer.process(code).unwrap();
    let mut vm = Vm::new(true);
    let result = vm.execute(program);
    match result {
        Err(e) => {
            println!("Error: {}", e.message);
            println!("===== Stack Trace =====");
            for item in e.stacktrace {
                println!("{}", item);
            }
            println!("===== App Stack =====");
            for item in e.app_stack {
                println!("{}", item);
            }
        }
        Ok(_) => (),
    }

    vm
}

#[cfg(test)]
mod test {
    use crate::run;

    #[test]
    fn counter_10000() {
        run(r#"
        section .code
            _main:
                call __dbg_print
                call __date_now_unix
                push r0
                mov ra,0
                mov rb,_loop_times
            _loop:
                call _print_name
                add ra,1
                test ra,rb
                jle _loop
                jmp _exit
            _print_name:
                mov rd,ra
                call __println
                ret
            _exit:
                call __date_now_unix
                pop r1
                sub r0,r1
                mov r1,r0
                mov rd,_took
                call __print
                mov rd,r1
                call __print
                test r1,1
                je _sec
            _secs:
                mov rd,_secs
                jmp _end
            _sec:
                mov rd,_sec
            _end:
                call __println
        section .data
            _space: " "
            _took: "Took "
            _secs: " seconds"
            _sec: " second"
            _loop_times: 10000
        "#
        .to_string());
    }

    #[test]
    fn alloc() {
        run(r#"
        section .code
            _main:
                alloc r0, 10
                mov r1, r0
                free r0
                call __dbg_print
        section .data
    
        "#
        .to_string());
    }

    #[test]
    fn can_assign_to_pointer() {
        let vm = run(r#"
            alloc ra, 10
            mov ra[0], 'y'
            mov ra[1], 'e'
            mov ra[2], 'y'
            ;free ra
            mov rd, ra
            call __println
        "#
        .to_string());

        let ra = &vm.registers.ra;
        assert_eq!("yey", ra.to_string());
        // let's free the pointer?
        let mut heap = crate::vm::heap::Heap::recover_poison(&vm.heap);
        let allocation = ra.to_p(&vm).unwrap();
        heap.deallocate(allocation.ptr, allocation.size).unwrap();
    }

    #[test]
    fn can_move_from_pointer_to_pointer() {
        let vm = run(r#"
            alloc ra, 4
            mov ra[0], 'd'
            mov ra[1], 'a'
            mov ra[2], 'y'
            alloc rb, 4
            mov rb[0], ra[0]
            mov rb[1], ra[1]
            mov rb[2], ra[2]
            mov rd, rb
            call __println
        "#
        .to_string());

        let rb = &vm.registers.rb;
        assert_eq!("day", rb.to_string());
        // let's free the pointer?
        let mut heap = crate::vm::heap::Heap::recover_poison(&vm.heap);
        let allocation = rb.to_p(&vm).unwrap();
        heap.deallocate(allocation.ptr, allocation.size).unwrap();
    }

    #[test]
    fn can_mov_string_offset_within_loop() {
        run(r#"
        section .code
            _main:
                mov ra, _len
                mov rb, _start
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

        section .data
            _start: 97
            _len: 26
        "#
        .to_string());
    }

    #[test]
    fn can_generate_random_string() {
        run(r#"
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
        section .data
            _chars: "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
            _len: 10
        "#
        .to_string());
    }
}
