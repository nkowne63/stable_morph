use std::io::stdin;

pub fn read_stdin() -> String {
    let mut lines = stdin().lines();
    let mut input = String::new();

    while let Some(line) = lines.next() {
        let line = line.unwrap();
        input += &(line + "\n");
    }

    input
}
