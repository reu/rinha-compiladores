use assert_cmd::Command;
use indoc::indoc;

macro_rules! rinha {
    ($expr:expr) => {{
        let ast = rinha::parser::parse_or_report("test.rinha", indoc! { $expr }).unwrap();
        let ast = serde_json::to_string_pretty(&ast).unwrap();

        let cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))
            .unwrap()
            .write_stdin(ast)
            .assert()
            .success();

        let out = cmd.get_output();
        let output = std::str::from_utf8(&out.stdout).unwrap();
        output.trim_end().to_owned()
    }};
}

#[test]
fn test_print() {
    assert_eq!(rinha!(r#"print("hello")"#), "hello");
    assert_eq!(rinha!("print(1 + 2)"), "3");
    assert_eq!(rinha!("print(1 == 1)"), "true");
    assert_eq!(rinha!("print(1 == 2)"), "false");
    assert_eq!(rinha!("print((1, false))"), "(1, false)");
    assert_eq!(
        rinha! { "
            let f = fn () => {
                1
            };
            print(f)
        "},
        "<#closure>"
    );
    assert_eq!(rinha!("print(print(1))"), "1\n1");
    assert_eq!(rinha!("print(\"1 + 2 = \" + 1 + 2)"), "1 + 2 = 3");
}

#[test]
fn test_binary_operations() {
    assert_eq!(rinha!("print(2 + 2)"), "4");
    assert_eq!(rinha!("print(2 - 4)"), "-2");
    assert_eq!(rinha!("print(2 * 4)"), "8");
    assert_eq!(rinha!("print(4 / 2)"), "2");
    assert_eq!(rinha!("print(6 % 4)"), "2");
    assert_eq!(rinha!("print((6 == 6, 6 == 5))"), "(true, false)");
    assert_eq!(rinha!("print((6 != 6, 6 != 5))"), "(false, true)");
    assert_eq!(rinha!("print((6 > 4, 3 > 5))"), "(true, false)");
    assert_eq!(rinha!("print((6 < 4, 3 < 5))"), "(false, true)");
    assert_eq!(rinha!("print((6 >= 6, 6 <= 5))"), "(true, false)");
    assert_eq!(rinha!("print((6 <= 6, 6 >= 5))"), "(true, true)");
    assert_eq!(rinha!("print(true && true)"), "true");
    assert_eq!(rinha!("print(true && false)"), "false");
    assert_eq!(rinha!("print(true || false)"), "true");
    assert_eq!(rinha!("print(false || false)"), "false");
}

#[test]
fn test_closure() {
    assert_eq!(
        rinha! {r#"
            let a = 1;
            let b = fn (n) => {
                let c = 2;
                let d = fn (n) => {
                    a + c + n
                };
                d(n)
            };
            print(b(1))
        "#},
        "4"
    );
}

#[test]
fn test_fibonacci() {
    assert_eq!(
        rinha! {r#"
            let fib = fn (n) => {
              if (n < 2) {
                n
              } else {
                fib(n - 1) + fib(n - 2)
              }
            };
            print("fib: " + fib(10))
        "#},
        "fib: 55"
    );
}

#[test]
fn test_currying() {
    assert_eq!(
        rinha! {r#"
            let add = fn (a) => {
              fn (b) => {
                a + b
              }
            };
            let addOne = add(1);
            print(addOne(2))
        "#},
        "3"
    );
}
