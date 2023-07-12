use std::process::{Termination, ExitCode};
use std::fmt::Debug;

use thistermination::{TerminationNoDebug};

#[derive(TerminationNoDebug)]
enum Test {
    UnitA,
    #[termination(exit_code(7))]
    UnitB,
    UnnamedA(u8),
    #[termination(exit_code(5))]
    NamedA{x: u32},
}

impl Debug for Test {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Test::UnitA => write!(f, "unit a"),
            Test::UnitB => write!(f, "unit b"),
            Test::UnnamedA(field) => write!(f, "unnamed {field}"),
            Test::NamedA { x } => write!(f, "named {}", x),
        }
    }
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
    let unit_b = Test::UnitB;
    assert_eq!(format!("{:?}", unit_a), "unit a");
    assert_eq!(format!("{:?}", unit_b), "unit b");
    assert_eq_exit_code_and_exit_code(unit_a.report(), ExitCode::FAILURE);
    assert_eq_exit_code_and_int(unit_b.report(), 7);
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
    assert_eq_exit_code_and_int(named.report(), 5);
}