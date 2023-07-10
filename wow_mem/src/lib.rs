#![allow(unused_imports)]

extern crate proc_macro;
use proc_macro::{TokenStream, Span};
use quote::{quote, quote_spanned, format_ident};
use syn::{parse_macro_input, DeriveInput, Token};

// type def_sub_46B840 = unsafe extern "thiscall" fn(u32) -> u32;

// static hook_sub_46B840: Lazy<GenericDetour<def_sub_46B840>> = Lazy::new(|| unsafe {
//     GenericDetour::new(
//         std::mem::transmute::<u32, def_sub_46B840>(0x0046B840),
//         sub_46B840,
//     )
//     .unwrap()
// });


#[proc_macro_attribute]
pub fn detour_fn(addr: TokenStream, item: TokenStream) -> TokenStream {
    let addr = parse_macro_input!(addr as syn::LitInt);
    let item = parse_macro_input!(item as syn::ItemFn);

    // println!("addr: \"{}\"", addr);
    // println!("item: \"{}\"", item.to_string());

    let sig = item.clone();
    let ident = &item.sig.ident;
    let def_ident = format_ident!("def_{}", item.sig.ident);
    let hook_ident = format_ident!("hook_{}", item.sig.ident);

    // FIXME: support a default abi (extern "C")
    let unsafety = item.sig.unsafety;
    let abi = item.sig.abi;
    let args = item.sig.inputs;
    let ret_type = item.sig.output;

    let expanded = quote! {
        type #def_ident = #unsafety #abi fn(#args) #ret_type;

        static #hook_ident: Lazy<GenericDetour<#def_ident>> = Lazy::new(|| unsafe {
            GenericDetour::new(
                std::mem::transmute::<u32, #def_ident>(#addr),
                #ident
            )
            .unwrap()
        });

        #sig
    };

    println!("\n\n{}\n\n\n", expanded.to_string());

    TokenStream::from(expanded)
}
