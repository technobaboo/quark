use heck::ToSnakeCase;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parse;
use syn::punctuated::{Pair, Punctuated};
use syn::token::Bracket;
use syn::{
    bracketed, parse_macro_input, parse_quote, Block, FnArg, Ident, ItemFn, ItemStruct, LitStr,
    Pat, Path, Signature, Token, Type,
};

#[proc_macro_attribute]
pub fn openxr(attr: TokenStream, item: TokenStream) -> TokenStream {
    let func: ItemFn = syn::parse(item).expect("Failed to parse function");
    let name = func.sig.ident.clone();
    let args: Punctuated<Pat, Token![,]> = func
        .sig
        .inputs
        .pairs()
        .map(|arg| match arg {
            Pair::Punctuated(FnArg::Typed(arg), p) => Pair::Punctuated(*arg.pat.clone(), *p),
            Pair::End(FnArg::Typed(arg)) => Pair::End(*arg.pat.clone()),
            _ => panic!("Expected function but recieved method"),
        })
        .collect();
    let new_func = ItemFn {
        sig: Signature {
            abi: Some(parse_quote!(extern "system")),
            ident: syn::parse(attr).expect("You must specify a function name"),
            output: parse_quote!(-> openxr::sys::Result),
            ..func.sig.clone()
        },
        block: Box::new(Block {
            stmts: vec![parse_quote! {
                match #name(#args) {
                    Ok(_) => openxr::sys::Result::SUCCESS,
                    Err(e) => e,
                }
            }],
            ..*func.block
        }),
        ..func.clone()
    };
    quote! {
        #[no_mangle]
        #[allow(non_snake_case)]
        #new_func

        #func
    }
    .into()
}

#[proc_macro_attribute]
pub fn handle(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item: ItemStruct = syn::parse(item).unwrap();
    let attr: Type = syn::parse(attr).unwrap();
    let ty = item.ident.clone();
    let mod_name = Ident::new(&item.ident.to_string().to_snake_case(), item.ident.span());
    quote! {
        #item

        mod #mod_name {
	        #[doc(hidden)]
	        static REGISTRY: std::sync::LazyLock<dashmap::DashMap<u64, super::#ty, std::hash::BuildHasherDefault<rustc_hash::FxHasher>>> = std::sync::LazyLock::new(|| std::default::Default::default());
	        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

	        #[doc(hidden)]
	        impl quark::handle::HandleData for super::#ty {
				type Handle = #attr;

	            fn registry<'a>() -> &'a dashmap::DashMap<u64, Self, std::hash::BuildHasherDefault<rustc_hash::FxHasher>> {
	                &REGISTRY
	            }
	            fn counter<'a>() -> &'a std::sync::atomic::AtomicU64 {
	                &COUNTER
	            }
	        }
        }
    }.into()
}

struct OxrFns {
    name: Ident,
    no_inst: Array<Path>,
    inst: Array<Path>,
    extensions: Array<NamedArray<Path>>,
}

impl Parse for OxrFns {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            name: input.parse()?,
            no_inst: input.parse()?,
            inst: input.parse()?,
            extensions: input.parse()?,
        })
    }
}

struct Array<T> {
    _bracket: Bracket,
    items: Punctuated<T, Token![,]>,
}

impl<T: Parse> Parse for Array<T> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            _bracket: bracketed!(content in input),
            items: content.parse_terminated(T::parse, Token![,])?,
        })
    }
}

struct NamedArray<T> {
    name: LitStr,
    _colon_token: Token![:],
    array: Array<T>,
}

impl<T: Parse> Parse for NamedArray<T> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            name: input.parse()?,
            _colon_token: input.parse()?,
            array: input.parse()?,
        })
    }
}

#[proc_macro]
pub fn oxr_fns(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as OxrFns);
    let name = input.name;
    let inst = input.inst.items.iter();
    let inst_str = inst
        .clone()
        .map(|f| f.segments.last().unwrap().ident.clone());
    let no_inst = input.no_inst.items.iter();
    let no_inst_str = no_inst
        .clone()
        .map(|f| f.segments.last().unwrap().ident.clone());
    let extensions = input.extensions.items.iter().map(|f| {
        let val = f.name.value();
        let val = val.as_str();
        let items = f.array.items.iter().map(|f| {
            let f_str = f.segments.last().unwrap().ident.clone();
            quote!(
                #[cfg(feature = #val)]
                (stringify!(#f_str), Ok(instance)) if instance.extensions.contains(&std::string::String::from(#val)) => Ok(unsafe { std::mem::transmute(#f as usize)}),
            )
        });
        quote!(
            #(
                #items
            )*
        )
    });
    quote! {
        fn #name(instance: openxr::sys::Instance, name: &str) -> std::result::Result<openxr::sys::pfn::VoidFunction, openxr::sys::Result> {
            match (name, instance.get()) {
                #(
                    (stringify!(#no_inst_str), _) => Ok(unsafe { std::mem::transmute(#no_inst as usize)}),
                )*
                (_, Err(_)) => Err(openxr::sys::Result::ERROR_HANDLE_INVALID),
                #(
                    (stringify!(#inst_str), Ok(_)) => Ok(unsafe { std::mem::transmute(#inst as usize)}),
                )*
                #(
                    #extensions
                )*
                (_, Ok(_)) => Err(openxr::sys::Result::ERROR_FUNCTION_UNSUPPORTED),
            }
        }
    }.into()
}
