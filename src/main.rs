use crate::lexer::lexer::Lexer;
use crate::vm::vm::Vm;

mod lexer;
mod vm;

fn main() {
    let lexer = Lexer::new();
    let val = lexer.process(r#"
.data
    @asuh "this tests storage in data"
    @wot 4
.code
        push @wot
        push 4
        add
        print
        call @name
        push "hi"
        print
        push @asuh
        print
        jmp @end
    @name
        push "my function!"
        print
        ret
    @end
        push "end of program"
        print
    "#.to_string());

    //println!("{:?}", val);
    let mut vm = Vm::new();
    vm.execute(val.unwrap());
}
