#![feature(layout_for_ptr)]

use vm::program::Program;

use crate::lexer::lexer::Lexer;
use crate::vm::vm::Vm;

mod lexer;
mod types;
mod vm;

fn counter_10000(lexer: Lexer) -> Option<Program> {
    lexer.process(
        r#"
    section .code
        _main:
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
        .to_string(),
    )
}

fn alloc_test(lexer: Lexer) -> Option<Program> {
    lexer.process(
        r#"
    section .code
        _main:
            alloc r0,10
            free r0
            call __dbg_print
    section .data

    "#
        .to_string(),
    )
}

fn main() {
    let lexer = Lexer::new();

    //let program = counter_10000(lexer).unwrap();
    let program = alloc_test(lexer).unwrap();

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
}
