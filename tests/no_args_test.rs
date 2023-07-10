use std::process::{Termination, ExitCode};

use thistermination::Termination;

//TODO: Tests
// without exitcode or message
// with thiserror

#[derive(Termination)]
enum Test {
    #[termination(exit_code(1), msg("unit a"))]
    UnitA,
    #[termination(exit_code(2), msg("unit b"))]
    UnitB(),
    #[termination(exit_code(3), msg("unnamed a"))]
    UnnamedA(u8),
    #[termination(exit_code(4), msg("named a"))]
    NamedA{x: u32},
}

fn assert_eq_exit_code_and_int(ex: ExitCode, code: i32) {
    assert_eq!(format!("{:?}", ex), format!("ExitCode(unix_exit_status({}))", code));
}

#[test]
fn unit_no_args() {
    let unit_a = Test::UnitA;
    let unit_b = Test::UnitB();
    assert_eq!(format!("{:?}", unit_a), "unit a");
    assert_eq!(format!("{:?}", unit_b), "unit b");
    assert_eq_exit_code_and_int(unit_a.report(), 1);
    assert_eq_exit_code_and_int(unit_b.report(), 2);
}

#[test]
fn unnamed_no_args() {
    let unnamed = Test::UnnamedA(42);
    assert_eq!(format!("{:?}", unnamed), "unnamed a");
    assert_eq_exit_code_and_int(unnamed.report(), 3);
}

#[test]
fn named_no_args() {
    let named = Test::NamedA{x: 1337};
    assert_eq!(format!("{:?}", named), "named a");
    assert_eq_exit_code_and_int(named.report(), 4);
}