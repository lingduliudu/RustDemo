use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

#[proc_macro_attribute]
pub fn log(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let name = input.sig.ident.to_string();
    let block = input.block;
    let vis = input.vis;
    let sig = input.sig;
    let expanded = quote! {
        #vis #sig {
            log::info!("{} called", #name);
            #block
        }
    };
    expanded.into()
}
