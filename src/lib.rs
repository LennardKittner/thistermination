use proc_macro::TokenStream;
use termination::_derive_termination;
use termination_no_debug::_derive_termination_no_debug;

mod termination;
mod termination_no_debug;
mod code_generation;
mod parse;

#[derive(PartialEq)]
enum DeriveMacroMode {
    Termination,
    TerminationNoDebug,
    TerminationFull,
}

//TODO: maybe another one containing all traits needed i.e. Display and Error and #[from]
#[proc_macro_derive(Termination, attributes(termination))]
pub fn derive_termination(steam: TokenStream) -> TokenStream {
   _derive_termination(steam)
}

#[proc_macro_derive(TerminationNoDebug, attributes(termination))]
pub fn derive_termination_no_debug(steam: TokenStream) -> TokenStream {
    _derive_termination_no_debug(steam)
}