use std::process::{Termination, ExitCode};

use thistermination::Termination;

#[derive(Termination)]
enum TestNoArgs {
    #[termination(exit_code(2), msg("unit {}", "test"))]
    UnitA,
    #[termination(exit_code(3), msg("unnamed {}", 69))]
    UnnamedA(u8),
    #[termination(exit_code(4), msg("named {} {}", "asdf", 5))]
    NamedA{x: u32},
}

fn assert_eq_exit_code_and_int(ex: ExitCode, code: i32) {
    assert_eq!(format!("{:?}", ex), format!("ExitCode(unix_exit_status({}))", code));
}

#[test]
fn unit_no_args() {
    let unit_a = TestNoArgs::UnitA;
    assert_eq!(format!("{:?}", unit_a), "unit test");
    assert_eq_exit_code_and_int(unit_a.report(), 2);
}

#[test]
fn unnamed_no_args() {
    let unnamed = TestNoArgs::UnnamedA(42);
    assert_eq!(format!("{:?}", unnamed), "unnamed 69");
    assert_eq_exit_code_and_int(unnamed.report(), 3);
}

#[test]
fn named_no_args() {
    let named = TestNoArgs::NamedA{x: 1337};
    assert_eq!(format!("{:?}", named), "named asdf 5");
    assert_eq_exit_code_and_int(named.report(), 4);
}