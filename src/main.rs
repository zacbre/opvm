#![feature(layout_for_ptr)]

use vm::program::Program;

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
    fn assign_to_pointer() {
        let vm = run(r#"
        section .code
            _main:
                alloc ra, 10
                call __dbg_heap
                mov ra[2], 5
                call __dbg_heap
                mov r3, ra[2]
                call __dbg_print
                call __dbg_heap
                free ra
        section .data
        "#
        .to_string());
        let r0 = vm.registers.r1;
    }
}
