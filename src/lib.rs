use proc_macro::TokenStream;
use termination::_derive_termination;
use termination_full::_derive_termination_full;
use termination_no_debug::_derive_termination_no_debug;

mod termination;
mod termination_no_debug;
mod termination_full;
mod code_generation;
mod parse;

#[proc_macro_derive(Termination, attributes(termination))]
pub fn derive_termination(steam: TokenStream) -> TokenStream {
   _derive_termination(steam)
}

#[proc_macro_derive(TerminationNoDebug, attributes(termination))]
pub fn derive_termination_no_debug(steam: TokenStream) -> TokenStream {
    _derive_termination_no_debug(steam)
}

//TODO: Display, Error and #[from]
#[proc_macro_derive(TerminationFull, attributes(termination, from))]
pub fn derive_termination_full(steam: TokenStream) -> TokenStream {
   _derive_termination_full(steam)
}