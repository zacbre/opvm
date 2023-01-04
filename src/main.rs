#![feature(layout_for_ptr)]

use crate::lexer::lexer::Lexer;
use crate::vm::vm::Vm;

mod lexer;
mod vm;

fn main() {
    let lexer = Lexer::new();
    let val = lexer.process(r#"
    #data
        .multi " loops!"
        .single " loop!"
        .looptimes 10                ; amount of times we are going to loop
        .done "done!"
    #code
        .main
            jmp @loop               ; jump to start of the program
        .exit
            mov r0,@done
            println r0
            ret
        .decideloopy
            mov rb,1                ; push 1 to compare counter
            test r0,rb              ; compare 1
            jl @multi               ; compare counter, if 1, print single
            mov rc,@single
            ret
        .multi
            mov rc,@multi
            ret
        .loop
            inc ra
            mov r0,ra
            call @decideloopy
            print ra
            println rc
            mov rb,@looptimes
            test rb,ra
            jl @loop
            jmp @alloc
        .alloc
            alloc ra,6
            mov ra[0] 'h'
            mov ra[1] 'e'
            mov ra[2] 'l'
            mov ra[3] 'l'
            mov ra[4] 'o'
            mov ra[5] 0x0a
            print ra
            free ra
            call @exit
    "#.to_string());

    let mut vm = Vm::new(true);
    let result = vm.execute(val.unwrap());
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
