use std::{process::{Termination, ExitCode}, num::ParseIntError, str::ParseBoolError};
use thistermination::TerminationFull;

//TODO: unit
//TODO: check display
#[derive(TerminationFull)]
enum Test {
    #[termination(msg("unit a"))]
    UnitA,
    #[termination(exit_code(3), msg("unnamed {0:?} {}", 69))]
    UnnamedA(u8),
    #[termination(exit_code(3), msg("Error: {0}"))]
    UnnamedB(#[from] ParseIntError),
    #[termination(exit_code(4), msg("Error: {err}"))]
    NamedA{#[from] err: ParseBoolError},
    #[termination(msg("named {} {y:?} {x}", "asdf"))]
    NamedB{x: u32, y: u8},
}

fn throw_error_1() -> Result<u32, Test> {
    let n: u32 = "abc".parse()?;
    Ok(n)
}

fn throw_error_2() -> Result<bool, Test> {
    let n: bool = "abc".parse()?;
    Ok(n)
}

fn assert_eq_exit_code_and_int(ex: ExitCode, code: i32) {
    assert_eq!(format!("{:?}", ex), format!("ExitCode(unix_exit_status({}))", code));
}

fn assert_eq_exit_code_and_exit_code(ex1: ExitCode, ex2: ExitCode) {
    assert_eq!(format!("{:?}", ex1), format!("{:?}", ex2));
}

#[test]
fn unit_full() {
    let unit_a = Test::UnitA;
    assert_eq!(format!("{:?}", unit_a), "unit a");
    assert_eq!(format!("{}", unit_a), "unit a");
    assert_eq_exit_code_and_exit_code(unit_a.report(), ExitCode::FAILURE);
}

#[test]
fn unnamed_full() {
    let unnamed_a = Test::UnnamedA(42);
    assert_eq!(format!("{:?}", unnamed_a), "unnamed 42 69");
    assert_eq!(format!("{}", unnamed_a), "unnamed 42 69");
    assert_eq_exit_code_and_int(unnamed_a.report(), 3);
    if let Err(unnamed_b) = throw_error_1() {
        assert_eq!(format!("{:?}", unnamed_b), "Error: invalid digit found in string");
        assert_eq!(format!("{}", unnamed_b), "Error: invalid digit found in string");
        assert_eq_exit_code_and_int(unnamed_b.report(), 3);
    } else {
        panic!()
    }
}

#[test]
fn named_full() {
    if let Err(named_a) = throw_error_2() {
        assert_eq!(format!("{:?}", named_a), "Error: provided string was not `true` or `false`");
        assert_eq!(format!("{}", named_a), "Error: provided string was not `true` or `false`");
        assert_eq_exit_code_and_int(named_a.report(), 4);
    } else {
        panic!()
    }
    let named_b = Test::NamedB{x: 2, y: 99};
    assert_eq!(format!("{:?}", named_b), "named asdf 99 2");
    assert_eq!(format!("{}", named_b), "named asdf 99 2");
    assert_eq_exit_code_and_exit_code(named_b.report(), ExitCode::FAILURE);
}