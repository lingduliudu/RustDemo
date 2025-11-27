use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg, ItemFn, Pat, ReturnType, Type};

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
#[proc_macro_attribute]
pub fn to_async(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);

    let vis = &func.vis;
    let sig = &func.sig;
    let block = &func.block;
    let name = &sig.ident;
    let inputs = &sig.inputs;
    let output = &sig.output;

    // inner 函数名
    let inner_name = syn::Ident::new(&format!("{}_inner", name), name.span());

    // ----------- 提取返回类型 T -------------
    let ret_type: Type = match output {
        ReturnType::Default => syn::parse_quote!(()),
        ReturnType::Type(_, ty) => (*ty.clone()),
    };

    // ----------- 提取参数名字 -------------
    let arg_idents: Vec<_> = inputs
        .iter()
        .map(|arg| match arg {
            FnArg::Typed(pat_type) => match &*pat_type.pat {
                Pat::Ident(ident) => ident.ident.clone(),
                _ => panic!("#[spawn_async] only supports simple identifier parameters"),
            },
            _ => panic!("#[spawn_async] does not support &self"),
        })
        .collect();

    // ----------- 生成代码 -------------
    let expanded = quote! {
        // 1. inner 函数包含原用户的函数体
        fn #inner_name(#inputs) -> #ret_type #block

        // 2. 对用户暴露的 spawn 包装
        #vis fn #name(#inputs) -> std::thread::JoinHandle<#ret_type> {
            std::thread::spawn(move || {
                #inner_name(#(#arg_idents),*)
            })
        }
    };

    TokenStream::from(expanded)
}
