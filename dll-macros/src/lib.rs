use proc_macro::TokenStream;
use proc_macro2::Literal;
use quote::{quote, ToTokens};
use syn::{
    self,
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote, Token,
};

struct Args {
    function: syn::Expr,
    args: syn::Expr,
    call_conv: syn::Expr,
    res_type: syn::Expr,
    arg_types: syn::Expr,
    cdecl: syn::Expr,
    stdcall: syn::Expr,
    ty_real: syn::Expr,
    ty_str: syn::Expr,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // i don't know if there's a better way to do this
        let function = input.parse()?;
        input.parse::<Token![,]>()?;
        let args = input.parse()?;
        input.parse::<Token![,]>()?;
        let call_conv = input.parse()?;
        input.parse::<Token![,]>()?;
        let res_type = input.parse()?;
        input.parse::<Token![,]>()?;
        let arg_types = input.parse()?;
        input.parse::<Token![,]>()?;
        let cdecl = input.parse()?;
        input.parse::<Token![,]>()?;
        let stdcall = input.parse()?;
        input.parse::<Token![,]>()?;
        let ty_real = input.parse()?;
        input.parse::<Token![,]>()?;
        let ty_str = input.parse()?;
        Ok(Args { function, args, call_conv, res_type, arg_types, cdecl, stdcall, ty_real, ty_str })
    }
}

#[proc_macro]
/// Call with setup_cvalue!(CValue) to create an enum CValue { Real(f64), Str(Rc<Vec<u8>>)}.
/// It comes with From<CValue> implementations for f64 and *const c_char.
pub fn setup_cvalue(tokens: TokenStream) -> TokenStream {
    let c_value = parse_macro_input!(tokens as syn::Ident);
    TokenStream::from(quote! {
        #[derive(Clone, Debug)]
        enum #c_value {
            Real(f64),
            Str(std::rc::Rc<std::vec::Vec<u8>>),
        }

        impl From<#c_value> for f64 {
            fn from(v: #c_value) -> Self {
                match v {
                    #c_value::Real(x) => x,
                    #c_value::Str(_) => 0.0,
                }
            }
        }

        impl From<#c_value> for *const std::os::raw::c_char {
            fn from(v: #c_value) -> Self {
                unsafe {
                    match v {
                        #c_value::Real(_) => b"\0\0\0\0\0".as_ptr().offset(4).cast(),
                        #c_value::Str(s) => s.as_ref().as_ptr().offset(4).cast(),
                    }
                }
            }
        }
    })
}

#[proc_macro]
/// This calls an external function with any possible combination of arguments.
/// Arguments are:
/// * The function pointer
/// * The arguments, as a vector of CValues
/// * The calling convention
/// * The result type
/// * The argument types
/// * The calling convention value corresponding to cdecl
/// * The calling convention value corresponding to stdcall
/// * The type value corresponding to reals
/// * The type value corresponding to strings
pub fn external_call(tokens: TokenStream) -> TokenStream {
    // unpack arguments
    let Args { function, args, call_conv, res_type, arg_types, cdecl, stdcall, ty_real, ty_str } =
        parse_macro_input!(tokens as Args);

    // generate an actual function call
    let generate_call = |abi, restype, argtypes: Vec<syn::BareFnArg>| -> syn::Expr {
        let mut fn_type: syn::TypeBareFn = parse_quote! {
            extern fn() -> f64
        };
        fn_type.abi = Some(abi);
        fn_type.inputs.extend(argtypes.iter().cloned());
        fn_type.output = syn::ReturnType::Type(Default::default(), restype);
        let func: syn::Expr = parse_quote! { std::mem::transmute::<_, #fn_type>(#function) };
        let mut call = syn::ExprCall {
            attrs: Vec::new(),
            func: Box::new(func),
            paren_token: Default::default(),
            args: Default::default(),
        };
        for i in 0..argtypes.len() {
            let int_expr: syn::LitInt = Literal::usize_unsuffixed(i).into();
            call.args.push(parse_quote! {#args[#int_expr].clone().into()});
        }
        parse_quote! {#call.into()}
    };

    // generate match on abi
    let match_abi = |restype: Box<syn::Type>, argtypes: Vec<syn::BareFnArg>| -> syn::Expr {
        let cdecl_call = generate_call(parse_quote! { extern "cdecl" }, restype.clone(), argtypes.clone());
        let stdcall_call = generate_call(parse_quote! { extern "stdcall" }, restype, argtypes);
        parse_quote! {
            match #call_conv {
                #cdecl => #cdecl_call,
                #stdcall => #stdcall_call,
            }
        }
    };

    // generate match on restype
    let match_restype = |argtypes: Vec<syn::BareFnArg>| -> syn::Expr {
        let real_call = match_abi(Box::new(parse_quote! {f64}), argtypes.clone());
        let str_call = match_abi(Box::new(parse_quote! {*const std::os::raw::c_char}), argtypes);
        parse_quote! {
            match #res_type {
                #ty_real => #real_call,
                #ty_str => #str_call,
            }
        }
    };

    // generate matches on argument types
    let mut argcount_matches: Vec<syn::Expr> = Vec::with_capacity(17);
    argcount_matches.push(match_restype(Vec::new()));
    for count in 1..5 {
        let mut arms: Vec<syn::Arm> = Vec::new();
        for comb in 0..(1 << count) {
            let mut argtypes: Vec<syn::BareFnArg> = vec![parse_quote! {f64}; count];
            let mut argenums = vec![ty_real.clone(); count];
            for arg_i in 0..count {
                if comb & (1 << arg_i) != 0 {
                    argtypes[arg_i] = parse_quote! {*const std::os::raw::c_char};
                    argenums[arg_i] = ty_str.clone();
                }
            }
            let mut pattern = syn::PatTuple {
                attrs: Vec::new(),
                paren_token: Default::default(),
                elems: argenums
                    .iter()
                    .map(|expr| syn::Pat::from(syn::PatLit { attrs: Vec::new(), expr: Box::new(expr.clone()) }))
                    .collect(),
            };
            if !pattern.elems.trailing_punct() {
                pattern.elems.push_punct(Default::default());
            }
            let body = match_restype(argtypes);
            arms.push(parse_quote! {#pattern => #body});
        }
        let mut arg_types = syn::ExprTuple {
            attrs: Vec::new(),
            paren_token: Default::default(),
            elems: (0..count)
                .map(|i| -> syn::Expr {
                    let int_expr: syn::LitInt = Literal::usize_unsuffixed(i).into();
                    parse_quote! {#arg_types[#int_expr]}
                })
                .collect(),
        };
        if !arg_types.elems.trailing_punct() {
            arg_types.elems.push_punct(Default::default());
        }
        argcount_matches.push(
            syn::ExprMatch {
                attrs: Vec::new(),
                match_token: Default::default(),
                expr: Box::new(arg_types.into()),
                brace_token: Default::default(),
                arms,
            }
            .into(),
        );
    }
    for count in 5..17 {
        argcount_matches.push(match_restype(vec![parse_quote! {f64}; count]));
    }

    // generate match arms for argcount
    let mut arms: Vec<syn::Arm> = argcount_matches
        .drain(..)
        .enumerate()
        .map(|(count, body)| {
            let int_expr: syn::LitInt = Literal::usize_unsuffixed(count).into();
            parse_quote! {#int_expr => #body}
        })
        .collect();

    arms.push(parse_quote! {_ => unimplemented!()});

    // return the final match
    syn::ExprMatch {
        attrs: Vec::new(),
        match_token: Default::default(),
        expr: parse_quote! {(#arg_types).len()},
        brace_token: Default::default(),
        arms,
    }
    .to_token_stream()
    .into()
}
