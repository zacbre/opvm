#![feature(layout_for_ptr)]

use crate::lexer::lexer::Lexer;
use crate::vm::vm::Vm;

mod lexer;
mod vm;

fn main() {
    let lexer = Lexer::new();
    let val = lexer.process(r#"
    section .code
        _main:
            call __date_now_unix
            mov r1,r0
            mov ra,0
            mov rb,_loop_times
        _loop:
            call _print_name
            inc ra
            test rb,ra
            jle _loop
            jmp _exit
        _print_name:
            mov rd,_wife_name
            mov re,_space
            call __concat
            mov rd,r0
            mov re,ra
            call __concat
            mov rd,r0
            call __println
            ret
        _exit:
            mov rd,_done
            call __print
            call __date_now
            mov rd,r0
            call __println
            call __date_now_unix
            sub r1,r0
            mov rd,_took
            call __print
            mov rd,r1
            call __println
    section .data
        _wife_name: "Katie"
        _space: " "
        _done: "Done at "
        _took: "Took "
        _loop_times: 100000
    "#.to_string());

    let program = val.unwrap();

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
        },
        Ok(_) => ()
    }
}
