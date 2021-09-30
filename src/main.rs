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
        .looptimes 5                ; amount of times we are going to loop
    #code
        .main
            push 0
            jmp @loop               ; jump to start of the program
        .decideloopy
            dup                     ; duplicate counter
            push 1                  ; push 1 to compare counter
            jg @multi               ; compare counter, if 1, print single
            push @single
            ret
        .multi
            push @multi
            ret
        .loop
            inc                     ; increment loop counter by 1
            dup                     ; duplicate loop counter for printing
            call @decideloopy
            concat                  ; concat what text we decided on
            print
            dup                     ; duplicate loop counter for comparing
            push @looptimes         ; push number of loops we expect
            jl @loop                ; less than expected, keep looping
    "#.to_string());

    let mut vm = Vm::new();
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
