use proc_macro::TokenStream;
use termination::_derive_termination;
use termination_no_debug::_derive_termination_no_debug;

mod termination;
mod termination_no_debug;
mod code_generation;
mod parse;

//TODO: maybe another one containing all traits needed i.e. Display and Error and #[from]
#[proc_macro_derive(Termination, attributes(termination))]
pub fn derive_termination(steam: TokenStream) -> TokenStream {
   _derive_termination(steam)
}

//TODO: error when msg is provided
#[proc_macro_derive(TerminationNoDebug, attributes(termination))]
pub fn derive_termination_no_debug(steam: TokenStream) -> TokenStream {
    _derive_termination_no_debug(steam)
}