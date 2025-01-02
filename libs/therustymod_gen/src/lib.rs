use proc_macro;
use std::sync::Mutex;
use self::proc_macro::TokenStream;
use std::str::FromStr;
use lazy_static::lazy_static;
use proc_macro2::{TokenStream as TokenStream2, Ident};

use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::{parse_macro_input, Data, DataStruct, parse_str, FnArg, Item, ItemMod, Fields, LitBool, Token, LitStr};

struct TRMDefinition {
    module_name: String,
    module_location: String,
}

lazy_static! {
    static ref TRM_DEFINITION: Mutex<Option<TRMDefinition>> = Mutex::new(None);
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
    let mut input = parse_macro_input!(input as syn::ItemMod);
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
    let annotated_signatures = signatures.clone().map(|sig| {
        quote! {
            #sig
        }
    });
    let bodies = funcs.zip(signatures.clone()).map(|(fdef, sig)| {
        let func = fdef.clone();
        let body = func.block;
        assert!(func.attrs.len() == 0, "TheRustyMod module must have functions with no attributes");
        quote! {
            #[no_mangle]
            extern "C" #sig #body
        }
    });

    let quoted = quote! {

        use therustymod::runtime::{vtable, VPtr};

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
            use therustymod::runtime::{TRM_SYSTEM as TRM_SYSTEM_, TRMSystem, TRMModuleData};
            use super::__run;
            lazy_static! {
                static ref TRM_MODULE_DATA: Arc<Mutex<TRMModuleData>> = Arc::new(Mutex::new(TRMModuleData {
                    module_name: CString::new(#module_name).unwrap(),
                    run: #daemon_ref
                }));
                pub static ref TRM_READY: () = {
                    let mut trm_system = TRM_SYSTEM_.lock().unwrap();
                    print!("Launching TRM system");
                    trm_system._set_module_data_once(TRM_MODULE_DATA.clone());
                };
            }
        }
    };

    // panic!("{}", quoted);

    TokenStream::from(quoted)
}
