use std::process::{Termination, ExitCode};

use thistermination::Termination;

#[derive(Termination)]
enum Test {
    #[termination(exit_code(3), msg("unnamed {0:?} {}", 69))]
    UnnamedA(u8),
    #[termination(exit_code(3), msg("unnamed {0} {:?} {1}", 69))]
    UnnamedB(u8, i32),
    #[termination(exit_code(4), msg("named {} {x}", "asdf"))]
    NamedA{x: u32},
    #[termination(exit_code(4), msg("named {} {y:?} {x}", "asdf"))]
    NamedB{x: u32, y: u8},
}

fn assert_eq_exit_code_and_int(ex: ExitCode, code: i32) {
    assert_eq!(format!("{:?}", ex), format!("ExitCode(unix_exit_status({}))", code));
}

#[test]
fn unnamed_mixed_args() {
    let unnamed_a = Test::UnnamedA(42);
    assert_eq!(format!("{:?}", unnamed_a), "unnamed 42 69");
    assert_eq_exit_code_and_int(unnamed_a.report(), 3);
    let unnamed_b = Test::UnnamedB(42, 7);
    assert_eq!(format!("{:?}", unnamed_b), "unnamed 42 69 7");
    assert_eq_exit_code_and_int(unnamed_b.report(), 3);
}

#[test]
fn named_mixed_args() {
    let named_a = Test::NamedA{x: 1337};
    assert_eq!(format!("{:?}", named_a), "named asdf 1337");
    assert_eq_exit_code_and_int(named_a.report(), 4);
    let named_b = Test::NamedB{x: 2, y: 99};
    assert_eq!(format!("{:?}", named_b), "named asdf 99 2");
    assert_eq_exit_code_and_int(named_b.report(), 4);
}