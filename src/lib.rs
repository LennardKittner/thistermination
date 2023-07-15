use proc_macro::TokenStream;
use termination::_derive_termination;
use termination_full::_derive_termination_full;
use termination_no_debug::_derive_termination_no_debug;

mod termination;
mod termination_no_debug;
mod termination_full;
mod code_generation;
mod parse;

//TODO: use parse_macro_input!
#[proc_macro_derive(Termination, attributes(termination))]
pub fn derive_termination(steam: TokenStream) -> TokenStream {
    match _derive_termination(steam) {
        Ok(stream) => stream,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(TerminationNoDebug, attributes(termination))]
pub fn derive_termination_no_debug(steam: TokenStream) -> TokenStream {
    match _derive_termination_no_debug(steam) {
        Ok(stream) => stream,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(TerminationFull, attributes(termination, from))]
pub fn derive_termination_full(steam: TokenStream) -> TokenStream {
    match _derive_termination_full(steam) {
        Ok(stream) => stream,
        Err(err) => err.to_compile_error().into(),
    }
}