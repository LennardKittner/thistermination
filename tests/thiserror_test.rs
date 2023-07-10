use std::process::{Termination, ExitCode};
use thiserror::Error;

use thistermination::Termination;

#[derive(Termination, Error)]
enum Test {
    #[error("unit a")]
    UnitA,
    #[error("unit b")]
    UnitB(),
    #[error("unit c")]
    #[termination(exit_code(7))]
    UnitC,
    #[error("unit d")]
    #[termination(msg("test"))]
    UnitD,
    #[error("unnamed {0}")]
    UnnamedA(u8),
    #[error("named {x}")]
    NamedA{x: u32},
}

fn assert_eq_exit_code_and_int(ex: ExitCode, code: i32) {
    assert_eq!(format!("{:?}", ex), format!("ExitCode(unix_exit_status({}))", code));
}

fn assert_eq_exit_code_and_exit_code(ex1: ExitCode, ex2: ExitCode) {
    assert_eq!(format!("{:?}", ex1), format!("{:?}", ex2));
}

#[test]
fn unit_default() {
    let unit_a = Test::UnitA;
    let unit_b = Test::UnitB();
    let unit_c = Test::UnitC;
    let unit_d = Test::UnitD;
    assert_eq!(format!("{:?}", unit_a), "unit a");
    assert_eq!(format!("{:?}", unit_b), "unit b");
    assert_eq!(format!("{:?}", unit_c), "unit c");
    assert_eq!(format!("{:?}", unit_d), "test");
    assert_eq_exit_code_and_exit_code(unit_a.report(), ExitCode::FAILURE);
    assert_eq_exit_code_and_exit_code(unit_b.report(), ExitCode::FAILURE);
    assert_eq_exit_code_and_int(unit_c.report(), 7);
    assert_eq_exit_code_and_exit_code(unit_d.report(), ExitCode::FAILURE);
}

#[test]
fn unnamed_default() {
    let unnamed = Test::UnnamedA(42);
    assert_eq!(format!("{:?}", unnamed), "unnamed 42");
    assert_eq_exit_code_and_exit_code(unnamed.report(), ExitCode::FAILURE);
}

#[test]
fn named_default() {
    let named = Test::NamedA{x: 1337};
    assert_eq!(format!("{:?}", named), "named 1337");
    assert_eq_exit_code_and_exit_code(named.report(), ExitCode::FAILURE);
}