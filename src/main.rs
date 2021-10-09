use crate::lexer::lexer::Lexer;
use crate::vm::vm::Vm;

mod lexer;
mod vm;

fn main() {
    let lexer = Lexer::new();
    let val = lexer.process(r#"
    push "What is your name? "
    print
    input
    push "Hi, "
    swap
    concat
    println
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
