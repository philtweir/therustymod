use proc_macro;
use std::sync::Mutex;
use self::proc_macro::TokenStream;
use std::collections::HashMap;
use std::str::FromStr;
use lazy_static::lazy_static;
use proc_macro2::{TokenStream as TokenStream2, Ident, Span};

use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::{parse_macro_input, ReturnType, Data, DataStruct, parse_str, FnArg, Item, ItemMod, Fields, LitBool, Token, LitStr, Type};

struct TRMDefinition {
    module_name: String,
    module_location: String,
}

lazy_static! {
    static ref TRM_DEFINITION: Mutex<Option<TRMDefinition>> = Mutex::new(None);
    static ref return_wrappers: HashMap<&'static str, (&'static str, &'static str)> = HashMap::from([
        ("bool", ("(TRM_SYSTEM.lock().unwrap().return_functions.as_ref().unwrap().return_int)(if ", " { 1 } else { 0 })")),
        // TODO: check safety
        ("*const c_char", ("(TRM_SYSTEM.lock().unwrap().return_functions.as_ref().unwrap().return_string)(", ")")),
        ("c_float", ("(TRM_SYSTEM.lock().unwrap().return_functions.as_ref().unwrap().return_float)(", ")")),
        ("c_int", ("(TRM_SYSTEM.lock().unwrap().return_functions.as_ref().unwrap().return_int)(", ")")),
        ("idVec3", ("(TRM_SYSTEM.lock().unwrap().return_functions.as_ref().unwrap().return_vector)(", ")")),
        ("idEntity", ("(TRM_SYSTEM.lock().unwrap().return_functions.as_ref().unwrap().return_entity((", ")")),
    ]);
}


struct CallbackArgs {
    daemon: bool,
}

mod keyword {
    syn::custom_keyword!(daemon);
    syn::custom_keyword!(name);
}

impl Parse for CallbackArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<keyword::daemon>()?;
        input.parse::<Token![=]>()?;
        let daemon: LitBool = input.parse()?;

        Ok(CallbackArgs {
            daemon: daemon.value(),
        })
    }
}

#[proc_macro]
pub fn therustymod_module_name(input: TokenStream) -> TokenStream {
    let definition = TRM_DEFINITION.lock().unwrap();
    // assert!(definition.is_some(), "therustymod_lib macro must be called before the runtime is imported (do not import the runtime manually");
    let quoted = if definition.is_none() {
        quote! ("")
    } else {
        let module_name = definition.as_ref().unwrap().module_name.clone();
        quote! {
            #module_name
        }
    };

    TokenStream::from(quoted)
}

#[proc_macro_attribute]
pub fn therustymod_lib(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as CallbackArgs);
    let input = parse_macro_input!(input as syn::ItemMod);
    // input.ident = Ident::new(name.as_str(), sig.ident.span());

    let expect_daemon = args.daemon;
    let struct_name = &input.ident;
    let module_name = struct_name.to_string();
    {
        let mut definition = TRM_DEFINITION.lock().unwrap();
        assert!(definition.is_none(), "therustymod_lib macro must be used only once");

        let module_name = struct_name.to_string();
        *definition = Some(TRMDefinition { module_name: module_name, module_location: String::new() });
    }

    let definitions = input.content.as_ref().unwrap().1.iter();
    let uses = definitions.clone().filter_map(|def| {
        match def {
            Item::Use(udef) => Some(quote! { #udef }),
            _ => None
        }
    });

    let funcs = definitions.clone().filter_map(|def| {
        match def {
            Item::Fn(fdef) => Some(fdef),
            Item::Use(_) => None,
            _ => panic!("TheRustyMod module must consist solely of functions and use statements")
        }
    });

    let run_daemon = funcs.clone().filter(|fdef| fdef.sig.ident.to_string() == "__run").next();

    assert!(expect_daemon == run_daemon.is_some(), "TheRustyMod must have a __run function iff. it is to run in daemon-mode");

    let (daemon, daemon_ref) = match run_daemon {
        Some(run_daemon) => (quote! {
            pub #run_daemon
        }, quote! {
            Some(Box::new(move || Box::pin(__run())))
        }),
        _ => (quote! { }, quote! { None })
    };

    let funcs = funcs.filter(|fdef| !fdef.sig.ident.to_string().starts_with("_"));
    let function_names = funcs.clone().map(|fdef| {
        let sig = fdef.clone().sig;
        (sig.ident.to_string(), format!("trm__usermod__{}__{}", struct_name, sig.ident))
    });
    let signatures = funcs.clone().zip(function_names.clone()).map(|(fdef, (_, name))| {
        let mut sig = fdef.clone().sig;
        let self_param = parse_str("&self").unwrap();
        sig.inputs.insert(0, self_param);
        sig.ident = Ident::new(name.as_str(), sig.ident.span());
        sig
    });
    let bodies = funcs.zip(signatures.clone()).map(|(fdef, mut sig)| {
        let func = fdef.clone();
        let body = func.block;
        let output = sig.output.clone();
        sig.output = syn::ReturnType::Default;
        assert!(func.attrs.len() == 0, "TheRustyMod module must have functions with no attributes");
        match output {
            ReturnType::Default => {
                quote! {
                    #[no_mangle]
                    extern "C" #sig #body
                }
            },
            ReturnType::Type(_, typ) => {
                let typ = typ.as_ref();
                let return_wrapper = match typ {
                    syn::Type::Path(path) => {
                        assert!(path.qself.is_none(), "Cannot have self return type");
                        let path = path.path.clone();
                        assert!(path.leading_colon.is_none(), "Cannot have :: in return type");
                        assert!(path.segments.len() == 1, "Must have one path segment in return type");
                        let typ = path.segments[0].ident.to_string();
                        return_wrappers.get(typ.as_str()).expect(format!("Could not find type: {}", typ).as_str())
                    },
                    syn::Type::Ptr(ptr) => {
                        assert!(ptr.const_token.is_some(), "Must have const in pointer return for *const c_char");
                        let typ = ptr.elem.as_ref();
                        match typ {
                            syn::Type::Path(path) => {
                                assert!(path.qself.is_none(), "Cannot have self return type");
                                let path = path.path.clone();
                                assert!(path.leading_colon.is_none(), "Cannot have :: in return type");
                                assert!(path.segments.len() == 1, "Must have one path segment in return type");
                                let typ = path.segments[0].ident.to_string();
                                assert!(typ == "c_char", "Can only have c_char as a *const return type");
                                return_wrappers.get("*const c_char").unwrap()
                            },
                            _ => panic!("Unknown *const return type")
                        }
                    },
                    _ => panic!("Unknown return type")
                };
                let return_wrapper: syn::Expr = parse_str(format!("{}result{}", return_wrapper.0, return_wrapper.1).as_str()).expect(format!("Could not construct return function: {}result{}", return_wrapper.0, return_wrapper.1).as_str());
                quote! {
                    #[no_mangle]
                    extern "C" #sig {
                        let result = #body;
                        unsafe { #return_wrapper }
                    }
                }
            }
        }
    });
    let annotated_signatures = signatures.clone().map(|mut sig| {
        sig.output = syn::ReturnType::Default;
        quote! {
            #sig
        }
    });

    let quoted = quote! {

        use therustymod::runtime::{vtable, VPtr, TRM_SYSTEM};

        #( #uses );*

        #[vtable]
        pub trait TRMUserIdClassVmt {
            #(
                #annotated_signatures
            );*;
        }

        #[derive(Default)]
        #[repr(C)]
        struct TRMUserIdClass {
            vftable: VPtr<dyn TRMUserIdClassVmt, Self>,
        }

        impl TRMUserIdClassVmt for TRMUserIdClass {
            #(
                #bodies
            )*
        }

        use ctor::ctor;
        #[ctor]
        fn #struct_name() {
            *#struct_name::TRM_READY;
        }

        #daemon

        mod #struct_name {
            use std::ffi::CString;
            use std::sync::{Arc, Mutex};
            use lazy_static::lazy_static; // Allow separate imports
            use therustymod::runtime::{TRM_SYSTEM, TRMSystem, TRMModuleData};
            use super::__run;
            lazy_static! {
                static ref TRM_MODULE_DATA: Arc<Mutex<TRMModuleData>> = Arc::new(Mutex::new(TRMModuleData {
                    module_name: CString::new(#module_name).unwrap(),
                    run: #daemon_ref
                }));
                pub static ref TRM_READY: () = {
                    let mut trm_system = TRM_SYSTEM.lock().unwrap();
                    print!("Launching TRM system");
                    trm_system._set_module_data_once(TRM_MODULE_DATA.clone());
                };
            }
        }
    };

    // panic!("{}", quoted);

    TokenStream::from(quoted)
}
