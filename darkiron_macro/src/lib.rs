extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, parse_quote};

#[proc_macro_attribute]
pub fn detour_fn(addr: TokenStream, func: TokenStream) -> TokenStream {
    let addr = parse_macro_input!(addr as syn::LitInt);
    let func = parse_macro_input!(func as syn::ItemFn);
    let mut orig_fn = func.clone();

    let func_ident = format_ident!("{}", func.sig.ident);
    let def_ident = format_ident!("def_{}", func.sig.ident);
    let hook_ident = format_ident!("hook_{}", func.sig.ident);

    // FIXME: support a default abi (extern "C")
    let unsafety = func.sig.unsafety;
    let abi = func.sig.abi;
    let args = func.sig.inputs;
    let ret_type = func.sig.output;
    let vis = func.vis;

    orig_fn.block.stmts.insert(0, parse_quote! {
        #[allow(unused_macros)] 
        macro_rules! original {
            ($($args:expr),* $(,)?) => {
                unsafe {
                    (#hook_ident).disable().unwrap();
                    let ret = (#hook_ident).call($($args),*);
                    (#hook_ident).enable().unwrap();
                    ret
                }
            };
        }
    });

    let expanded = quote! {
        type #def_ident = #unsafety #abi fn(#args) #ret_type;

        #vis static #hook_ident: once_cell::sync::Lazy<retour::GenericDetour<#def_ident>> = once_cell::sync::Lazy::new(|| unsafe {
            retour::GenericDetour::new(
                std::mem::transmute::<u32, #def_ident>(#addr),
                #func_ident
            )
            .unwrap()
        });

        #orig_fn
    };

    // println!("\n\n{}\n\n\n", expanded.to_string());

    TokenStream::from(expanded)
}

/// # Usage
///
/// ```
/// #[hook_fn(0x0063CB50)]
/// extern "fastcall" fn ConsoleWriteRaw(text: *const c_char, color: ConsoleColor) {}
/// ```
#[proc_macro_attribute]
pub fn hook_fn(addr: TokenStream, def: TokenStream) -> TokenStream {
    let addr = parse_macro_input!(addr as syn::LitInt);
    let def = parse_macro_input!(def as syn::ItemFn);
    let orig_def = def.clone();

    let def_ident = format_ident!("def_{}", def.sig.ident);

    // FIXME: support a default abi (extern "C")
    let unsafety = def.sig.unsafety;
    let abi = def.sig.abi;
    let args = def.sig.inputs;
    let ret_type = def.sig.output;

    let sig = orig_def.sig;
    let vis = orig_def.vis;

    let value_iter = args.clone().into_iter();

    let mut values = syn::punctuated::Punctuated::<Box<syn::Pat>, syn::token::Comma>::new();

    for arg in value_iter {
        let ty = match arg {
            syn::FnArg::Receiver(_) => panic!("fuck"),
            syn::FnArg::Typed(ty) => ty,
        };

        values.push(ty.pat);
    }

    let expanded = quote! {
        type #def_ident = #unsafety #abi fn(#args) #ret_type;

        #vis #sig {
            let func = unsafe { std::mem::transmute::<u32, #def_ident>(#addr) };
            func(#values)
        }
    };

    // println!("\n\n{}\n\n\n", expanded.to_string());

    TokenStream::from(expanded)
}
