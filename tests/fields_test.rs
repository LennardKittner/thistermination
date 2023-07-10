use std::process::{Termination, ExitCode};

use thistermination::Termination;

#[derive(Termination)]
enum TestNoArgs {
    #[termination(exit_code(3), msg("unnamedA {0}"))]
    UnnamedA(u8),
    #[termination(exit_code(3), msg("unnamedB {0} {1}"))]
    UnnamedB(u8, i32),
    #[termination(exit_code(4), msg("namedA {x}"))]
    NamedA{x: u32},
    #[termination(exit_code(4), msg("namedB {x} {y}"))]
    NamedB{x: u32, y: u8},
}

fn assert_eq_exit_code_and_int(ex: ExitCode, code: i32) {
    assert_eq!(format!("{:?}", ex), format!("ExitCode(unix_exit_status({}))", code));
}


#[test]
fn unnamed_field_args() {
    let unnamed = TestNoArgs::UnnamedA(42);
    assert_eq!(format!("{:?}", unnamed), "unnamedA 42");
    assert_eq_exit_code_and_int(unnamed.report(), 3);
    let unnamed = TestNoArgs::UnnamedB(42, 55);
    assert_eq!(format!("{:?}", unnamed), "unnamedB 42 55");
    assert_eq_exit_code_and_int(unnamed.report(), 3);
}

#[test]
fn named_field_args() {
    let named = TestNoArgs::NamedA{x: 1337};
    assert_eq!(format!("{:?}", named), "namedA 1337");
    assert_eq_exit_code_and_int(named.report(), 4);
    let named = TestNoArgs::NamedB{x: 1337, y: 33};
    assert_eq!(format!("{:?}", named), "namedB 1337 33");
    assert_eq_exit_code_and_int(named.report(), 4);
}