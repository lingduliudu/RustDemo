use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg, ItemFn};

#[proc_macro_attribute]
pub fn totlog(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let vis = &input.vis;
    let sig = &input.sig;
    let block = &input.block;
    let fn_name = sig.ident.to_string();
    let params = attr.to_string();

    // 处理函数参数
    let args = sig.inputs.iter().map(|arg| match arg {
        FnArg::Typed(pat) => {
            let name = &pat.pat;
            quote! {
                format!("{}={:?}", stringify!(#name), #name).to_string()
            }
        }
        FnArg::Receiver(_) => quote! { "self".to_string() },
    });

    let async_token = &sig.asyncness;

    let gen = if async_token.is_some() {
        quote! {
            #vis #sig {
                use log::info;
                let args_vec: Vec<String> = vec![#(#args),*];
                let args_str = if args_vec.is_empty() { "".to_string() } else { args_vec.join(", ") };
                if !#params.trim().is_empty() {
                    info!("totlog 参数:{}",#params.trim());
                }
                info!("调用函数: {}({})", #fn_name, args_str);
                let __totlog_result = async move { #block }.await;
                info!("调用函数: {}({}) 返回: {:?}", #fn_name,args_str, __totlog_result);
                __totlog_result
            }
        }
    } else {
        quote! {
            #vis #sig {
                use log::info;
                let args_vec: Vec<String> = vec![#(#args),*];
                let args_str = if args_vec.is_empty() { "".to_string() } else { args_vec.join(", ") };
                if !#params.trim().is_empty() {
                    info!("totlog 参数:{}",#params.trim());
                }
                info!("调用函数: {}({})",now.format("%Y-%m-%d %H:%M:%S.%f"), #fn_name, args_str);
                let __totlog_result = { #block };
                info!("调用函数: {}({}) 返回: {:?}", #fn_name,args_str, __totlog_result);

                __totlog_result
            }
        }
    };

    gen.into()
}
