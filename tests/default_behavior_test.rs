use std::process::{Termination, ExitCode};

use thistermination::TerminationFull;

#[derive(TerminationFull)]
#[termination(exit_code(6), msg("test123 {}", 9 + 8))]
enum Test {
    #[termination(exit_code(7))]
    UnitA,
    #[termination(msg("test"))]
    UnnamedA(u8),
    NamedA{x: u32},
}

fn assert_eq_exit_code_and_int(ex: ExitCode, code: i32) {
    assert_eq!(format!("{:?}", ex), format!("ExitCode(unix_exit_status({}))", code));
}

#[test]
fn unit_default() {
    let unit = Test::UnitA;
    assert_eq!(format!("{:?}", unit), "test123 17");
    assert_eq!(format!("{}", unit), "test123 17");
    assert_eq_exit_code_and_int(unit.report(), 7);
}

#[test]
fn unnamed_default() {
    let unnamed = Test::UnnamedA(42);
    assert_eq!(format!("{:?}", unnamed), "test");
    assert_eq!(format!("{}", unnamed), "test");
    assert_eq_exit_code_and_int(unnamed.report(), 6);
}

#[test]
fn named_default() {
    let named = Test::NamedA{x: 1337};
    assert_eq!(format!("{:?}", named), "test123 17");
    assert_eq!(format!("{}", named), "test123 17");
    assert_eq_exit_code_and_int(named.report(), 6);
}