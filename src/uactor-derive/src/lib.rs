use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, FnArg, ImplItem, ImplItemFn, ItemImpl, Pat};

/// Derive macro for the `Message` trait. Implements `static_name()` returning the type name.
///
/// For enums, `name(&self)` also includes the variant: `"MyEnum::VariantA"`.
///
/// ```ignore
/// #[derive(uactor::Message)]
/// struct MyMessage { data: String }
///
/// #[derive(uactor::Message)]
/// enum MyEvent {
///     Created(u64),
///     Deleted { id: u64 },
///     Shutdown,
/// }
/// // MyEvent::static_name() => "MyEvent"
/// // MyEvent::Created(1).name() => "MyEvent::Created"
/// ```
#[proc_macro_derive(Message)]
pub fn derive_message(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let name_str = name.to_string();

    let name_method = match &input.data {
        syn::Data::Enum(data_enum) => {
            let arms = data_enum.variants.iter().map(|variant| {
                let variant_ident = &variant.ident;
                let full_name = format!("{}::{}", name_str, variant_ident);
                match &variant.fields {
                    syn::Fields::Unit => quote! { Self::#variant_ident => #full_name, },
                    syn::Fields::Unnamed(_) => quote! { Self::#variant_ident(..) => #full_name, },
                    syn::Fields::Named(_) => quote! { Self::#variant_ident { .. } => #full_name, },
                }
            });
            quote! {
                fn name(&self) -> String {
                    match self {
                        #(#arms)*
                    }.to_owned()
                }
            }
        }
        _ => quote! {},
    };

    let expanded = quote! {
        impl #impl_generics uactor::actor::message::Message for #name #ty_generics #where_clause {
            fn static_name() -> &'static str {
                #name_str
            }
            #name_method
        }
    };

    TokenStream::from(expanded)
}

/// Marker attribute for handler methods inside `#[uactor::actor]` impl blocks.
/// Must not be used standalone.
#[proc_macro_attribute]
pub fn handler(_attr: TokenStream, _item: TokenStream) -> TokenStream {
    TokenStream::from(quote! {
        compile_error!("`#[handler]` must be used inside an `#[uactor::actor]` impl block");
    })
}

/// Attribute macro for an actor impl block. Methods marked with `#[uactor::handler]`
/// are transformed into `impl Handler<MsgType>` trait implementations.
///
/// ## Parameter convention
///
/// - `&mut self` / `&self` (optional) -- access actor fields
/// - First non-self parameter -- the message; its type determines `Handler<Type>`
/// - `ctx` -- maps to `ctx: &mut Self::Context`
/// - `state` -- maps to `state: &Self::State`
/// - Any other parameter -- accessed as `inject.field_name` from the `Inject` struct
///
/// ## Example
///
/// ```ignore
/// #[uactor::actor]
/// impl MyActor {
///     #[uactor::handler]
///     async fn handle_ping(&mut self, PingMsg(reply): PingMsg, service: Service<MyService>) -> HandleResult {
///         service.do_work();
///         let _ = reply.send(PongMsg);
///         Ok(())
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn actor(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemImpl);
    let actor_type = input.self_ty.clone();

    let mut handler_impls = Vec::new();
    let mut remaining_items = Vec::new();

    for item in input.items.drain(..) {
        match item {
            ImplItem::Fn(method) if has_handler_attr(&method) => {
                handler_impls.push(generate_handler_impl(&actor_type, method));
            }
            other => remaining_items.push(other),
        }
    }

    input.items = remaining_items;

    let expanded = quote! {
        #input
        #(#handler_impls)*
    };

    TokenStream::from(expanded)
}

fn is_handler_path(attr: &syn::Attribute) -> bool {
    let segments: Vec<_> = attr.path().segments.iter().collect();
    match segments.len() {
        1 => segments[0].ident == "handler",
        2 => segments[0].ident == "uactor" && segments[1].ident == "handler",
        _ => false,
    }
}

fn has_handler_attr(method: &ImplItemFn) -> bool {
    method.attrs.iter().any(is_handler_path)
}

fn generate_handler_impl(
    actor_type: &syn::Type,
    mut method: ImplItemFn,
) -> proc_macro2::TokenStream {
    method.attrs.retain(|attr| !is_handler_path(attr));

    let remaining_attrs = &method.attrs;
    let stmts = &method.block.stmts;

    let params: Vec<_> = method
        .sig
        .inputs
        .iter()
        .filter(|arg| !matches!(arg, FnArg::Receiver(_)))
        .collect();

    if params.is_empty() {
        panic!("#[handler] method must have at least one parameter (the message type)");
    }

    let (msg_pat, msg_type) = match params[0] {
        FnArg::Typed(pat_type) => (&*pat_type.pat, &*pat_type.ty),
        _ => unreachable!(),
    };

    let mut inject_bindings = Vec::new();
    let mut has_ctx = false;
    let mut has_state = false;
    let mut ctx_ident = None;
    let mut state_ident = None;

    for param in &params[1..] {
        if let FnArg::Typed(pat_type) = param {
            if let Pat::Ident(pat_ident) = &*pat_type.pat {
                let name = &pat_ident.ident;
                if name == "ctx" || name == "context" {
                    has_ctx = true;
                    ctx_ident = Some(name.clone());
                } else if name == "state" {
                    has_state = true;
                    state_ident = Some(name.clone());
                } else {
                    inject_bindings.push(name.clone());
                }
            } else {
                panic!("#[handler] parameters (except message) must be simple identifiers");
            }
        }
    }

    let ctx_param = if has_ctx {
        let name = ctx_ident.unwrap();
        quote! { #name: &mut Self::Context }
    } else {
        quote! { _ctx: &mut Self::Context }
    };

    let state_param = if has_state {
        let name = state_ident.unwrap();
        quote! { #name: &Self::State }
    } else {
        quote! { _state: &Self::State }
    };

    let inject_lets: Vec<proc_macro2::TokenStream> = inject_bindings
        .iter()
        .map(|name| {
            quote! { let #name = &mut inject.#name; }
        })
        .collect();

    quote! {
        impl uactor::actor::abstract_actor::Handler<#msg_type> for #actor_type {
            #(#remaining_attrs)*
            async fn handle(
                &mut self,
                inject: &mut Self::Inject,
                #msg_pat: #msg_type,
                #ctx_param,
                #state_param,
            ) -> uactor::actor::abstract_actor::HandleResult {
                #(#inject_lets)*
                #(#stmts)*
            }
        }
    }
}
