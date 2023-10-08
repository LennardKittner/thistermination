use thistermination::{TerminationFull, Termination, TerminationNoDebug};

//The "test" will fail if this file does not compile

#[derive(TerminationFull)]
enum Test1 {}

#[derive(Termination)]
enum Test2 {}

#[derive(TerminationNoDebug)]
enum Test3 {}