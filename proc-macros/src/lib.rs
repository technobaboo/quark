use heck::ToSnakeCase;
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::parse::Parse;
use syn::punctuated::{Pair, Punctuated};
use syn::token::Bracket;
use syn::{
    bracketed, parse_macro_input, parse_quote, Block, FnArg, Ident, ItemFn, ItemStruct, LitStr,
    Pat, Path, Signature, Token, Type,
};

#[proc_macro_attribute]
pub fn wrap_openxr(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut func: ItemFn = syn::parse(item).expect("Failed to parse function");

    func.sig.abi = Some(parse_quote!(extern "system"));
    func.sig.output = parse_quote!(-> openxr::sys::Result);

    let block = &func.block;
    // Modify the function body
    func.block = Box::new(parse_quote! {{
        let result = (|| #block)();
        match result {
            Ok(_) => openxr::sys::Result::SUCCESS,
            Err(e) => e,
        }
    }});
    func.into_token_stream().into()
}

#[proc_macro_attribute]
pub fn export_openxr(attr: TokenStream, item: TokenStream) -> TokenStream {
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
    let attr_name = match &attr {
        Type::Path(type_path) => type_path.path.segments.last().unwrap().ident.to_string(),
        _ => panic!("Expected a path type"),
    };
    let ty = item.ident.clone();
    let destroy_fn_name = Ident::new(
        &format!("xr_destroy_{}", attr_name.to_snake_case()),
        ty.span(),
    );

    quote! {
        #item

        impl Handle<#ty> for #attr {
            fn from_raw(raw: u64) -> Self {
                Self::from_raw(raw)
            }
            fn into_raw(self) -> u64 {
                self.into_raw()
            }
        }

        #[quark::wrap_openxr]
        pub fn #destroy_fn_name(handle: #attr) -> XrResult {
            handle.remove_data();
            Ok(())
        }
    }
    .into()
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
