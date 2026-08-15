use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, parse_quote, Attribute, Error, FnArg, Ident, ImplItem, ImplItemFn, ItemImpl,
    Pat, Result, ReturnType, Signature, Type,
};

pub fn service(attr: TokenStream, item: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        return Error::new(
            proc_macro2::Span::call_site(),
            "#[uactor::service] does not take arguments",
        )
        .to_compile_error()
        .into();
    }

    let input = parse_macro_input!(item as ItemImpl);
    match expand_service(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn expand_service(mut input: ItemImpl) -> Result<TokenStream2> {
    if input.trait_.is_some() {
        return Err(Error::new_spanned(
            &input,
            "#[uactor::service] must be applied to an inherent `impl ActorType` block, not a trait impl",
        ));
    }

    let actor_ident = actor_ident(&input.self_ty)?;
    let actor_type = input.self_ty.clone();

    let mut remaining_items = Vec::new();
    let mut methods = Vec::new();

    for item in input.items.drain(..) {
        match item {
            ImplItem::Fn(method) if has_ident_attr(&method.attrs, "skip") => {
                let mut method = method;
                strip_helper_attrs(&mut method);
                remaining_items.push(ImplItem::Fn(method));
            }
            ImplItem::Fn(method) => methods.push(method),
            other => remaining_items.push(other),
        }
    }

    if methods.is_empty() {
        return Err(Error::new_spanned(
            &actor_type,
            "#[uactor::service] impl block must contain at least one method",
        ));
    }

    let mut message_defs = Vec::new();
    let mut handler_impls = Vec::new();
    let mut ref_methods = Vec::new();
    let mut message_idents = Vec::new();

    for method in &methods {
        let generated = generate_for_method(&actor_type, method)?;
        message_idents.push(generated.message_ident.clone());
        message_defs.push(generated.message_def);
        handler_impls.push(generated.handler_impl);
        ref_methods.push(generated.ref_method);
    }

    let mut kept_methods = methods;
    for method in &mut kept_methods {
        strip_helper_attrs(method);
    }
    remaining_items.extend(kept_methods.into_iter().map(ImplItem::Fn));
    input.items = remaining_items;

    let msg_enum = format_ident!("{}Msg", actor_ident);
    let actor_ref = format_ident!("{}Ref", actor_ident);

    Ok(quote! {
        #input

        #(#message_defs)*

        #(#handler_impls)*

        uactor::generate_actor_ref!(#actor_ident, { #(#message_idents),* });

        impl<T> #actor_ref<T>
        where
            T: uactor::data::data_publisher::DataPublisher<Item = #msg_enum> + Clone,
        {
            #(#ref_methods)*
        }

        const _: fn() = || {
            fn __uactor_assert_route_message<A>()
            where
                A: uactor::actor::abstract_actor::Actor<RouteMessage = #msg_enum>,
            {
            }
            __uactor_assert_route_message::<#actor_ident>();
        };
    })
}

struct GeneratedMethod {
    message_ident: Ident,
    message_def: TokenStream2,
    handler_impl: TokenStream2,
    ref_method: TokenStream2,
}

fn generate_for_method(actor_type: &Type, method: &ImplItemFn) -> Result<GeneratedMethod> {
    let method_ident = &method.sig.ident;
    let message_ident = Ident::new(
        &to_pascal_case(&method_ident.to_string()),
        method_ident.span(),
    );
    let kind = method_kind(method);
    let params = classify_params(&method.sig)?;
    let receiver = receiver_kind(&method.sig)?;
    let ret_ty: Type = match &method.sig.output {
        ReturnType::Default => parse_quote!(()),
        ReturnType::Type(_, ty) => ty.as_ref().clone(),
    };
    let is_async = method.sig.asyncness.is_some();

    if !method.sig.generics.params.is_empty() {
        return Err(Error::new_spanned(
            &method.sig.generics,
            "#[uactor::service] methods cannot have generic parameters",
        ));
    }

    let field_params: Vec<_> = params
        .iter()
        .filter(|p| matches!(p, Param::Field { .. }))
        .collect();

    let message_def = generate_message_struct(&message_ident, &field_params, kind, &ret_ty);
    let handler_impl = generate_handler(
        actor_type,
        method,
        &message_ident,
        &params,
        receiver,
        kind,
        is_async,
    );
    let ref_method = generate_ref_method(method, &message_ident, &field_params, kind, &ret_ty);

    Ok(GeneratedMethod {
        message_ident,
        message_def,
        handler_impl,
        ref_method,
    })
}

fn generate_message_struct(
    message_ident: &Ident,
    fields: &[&Param],
    kind: MethodKind,
    ret_ty: &Type,
) -> TokenStream2 {
    let named_fields: Vec<TokenStream2> = fields
        .iter()
        .map(|param| match param {
            Param::Field { ident, ty, .. } => quote! { pub #ident: #ty },
            _ => unreachable!(),
        })
        .collect();

    match kind {
        MethodKind::Send if named_fields.is_empty() => {
            quote! {
                #[derive(uactor::Message)]
                pub struct #message_ident;
            }
        }
        MethodKind::Send => {
            quote! {
                #[derive(uactor::Message)]
                pub struct #message_ident {
                    #(#named_fields,)*
                }
            }
        }
        MethodKind::Ask => {
            quote! {
                #[derive(uactor::Message)]
                pub struct #message_ident {
                    #(#named_fields,)*
                    pub reply: uactor::actor::message::Reply<#ret_ty>,
                }
            }
        }
    }
}

fn generate_handler(
    actor_type: &Type,
    method: &ImplItemFn,
    message_ident: &Ident,
    params: &[Param],
    receiver: ReceiverKind,
    kind: MethodKind,
    is_async: bool,
) -> TokenStream2 {
    let method_ident = &method.sig.ident;

    let field_idents: Vec<_> = params
        .iter()
        .filter_map(|p| match p {
            Param::Field { ident, .. } => Some(ident),
            _ => None,
        })
        .collect();

    let has_inject = params.iter().any(|p| matches!(p, Param::Inject { .. }));

    let ctx_param = match params.iter().find_map(|p| match p {
        Param::Ctx { ident } => Some(ident),
        _ => None,
    }) {
        Some(ident) => quote! { #ident: &mut Self::Context },
        None => quote! { _ctx: &mut Self::Context },
    };

    let state_param = match params.iter().find_map(|p| match p {
        Param::State { ident } => Some(ident),
        _ => None,
    }) {
        Some(ident) => quote! { #ident: &Self::State },
        None => quote! { _state: &Self::State },
    };

    let inject_param = if has_inject {
        quote! { inject: &mut Self::Inject }
    } else {
        quote! { _inject: &mut Self::Inject }
    };

    let call_args: Vec<TokenStream2> = params
        .iter()
        .map(|p| match p {
            Param::Field { ident, .. } => quote! { #ident },
            Param::Ctx { ident } => quote! { #ident },
            Param::State { ident } => quote! { #ident },
            Param::Inject { ident } => quote! { &mut inject.#ident },
        })
        .collect();

    let call_base = match receiver {
        ReceiverKind::None => quote! { <Self>::#method_ident(#(#call_args),*) },
        ReceiverKind::Ref | ReceiverKind::Mut => quote! { self.#method_ident(#(#call_args),*) },
    };

    let call = if is_async {
        quote! { #call_base.await }
    } else {
        call_base
    };

    let bind_msg = match (kind, field_idents.is_empty()) {
        (MethodKind::Send, true) => quote! { let _ = msg; },
        (MethodKind::Send, false) => quote! {
            let #message_ident { #(#field_idents,)* } = msg;
        },
        (MethodKind::Ask, _) => quote! {
            let #message_ident { #(#field_idents,)* reply } = msg;
        },
    };

    let handle_body = match kind {
        MethodKind::Send if is_plain_unit_return(&method.sig) => {
            quote! {
                #call;
                Ok(())
            }
        }
        MethodKind::Send => {
            quote! {
                #call.map_err(::core::convert::Into::into)
            }
        }
        MethodKind::Ask => {
            quote! {
                let __result = #call;
                let _ = reply.send(__result);
                Ok(())
            }
        }
    };

    quote! {
        impl uactor::actor::abstract_actor::Handler<#message_ident> for #actor_type {
            async fn handle(
                &mut self,
                #inject_param,
                msg: #message_ident,
                #ctx_param,
                #state_param,
            ) -> uactor::actor::abstract_actor::HandleResult {
                #bind_msg
                #handle_body
            }
        }
    }
}

fn generate_ref_method(
    method: &ImplItemFn,
    message_ident: &Ident,
    fields: &[&Param],
    kind: MethodKind,
    ret_ty: &Type,
) -> TokenStream2 {
    let method_ident = &method.sig.ident;
    let docs: Vec<_> = method
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("doc"))
        .collect();

    let args: Vec<TokenStream2> = fields
        .iter()
        .map(|p| match p {
            Param::Field { ident, ty, .. } => quote! { #ident: #ty },
            _ => unreachable!(),
        })
        .collect();

    let construct_fields: Vec<TokenStream2> = fields
        .iter()
        .map(|p| match p {
            Param::Field { ident, .. } => quote! { #ident },
            _ => unreachable!(),
        })
        .collect();

    match kind {
        MethodKind::Send => {
            let send_ident = format_ident!("send_{}", method_ident);
            let construct = if construct_fields.is_empty() {
                quote! { #message_ident }
            } else {
                quote! { #message_ident { #(#construct_fields,)* } }
            };
            quote! {
                #(#docs)*
                pub fn #send_ident(&self, #(#args),*) -> uactor::data::data_publisher::DataPublisherResult {
                    <Self as uactor::actor::abstract_actor::MessageSender<#message_ident>>::send(
                        self,
                        #construct,
                    )
                }
            }
        }
        MethodKind::Ask => {
            let ask_ident = format_ident!("ask_{}", method_ident);
            quote! {
                #(#docs)*
                pub async fn #ask_ident(
                    &self,
                    #(#args),*
                ) -> ::core::result::Result<
                    #ret_ty,
                    uactor::data::data_publisher::DataPublisherErrors,
                > {
                    <Self as uactor::actor::abstract_actor::MessageSender<#message_ident>>::ask(
                        self,
                        |reply| #message_ident { #(#construct_fields,)* reply },
                    )
                    .await
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
enum MethodKind {
    Send,
    Ask,
}

enum ReceiverKind {
    None,
    Ref,
    Mut,
}

enum Param {
    Field { ident: Ident, ty: Type },
    Ctx { ident: Ident },
    State { ident: Ident },
    Inject { ident: Ident },
}

fn method_kind(method: &ImplItemFn) -> MethodKind {
    if is_fire_and_forget_unit(&method.sig) || is_handle_result(&method.sig) {
        MethodKind::Send
    } else {
        MethodKind::Ask
    }
}

fn receiver_kind(sig: &Signature) -> Result<ReceiverKind> {
    match sig.receiver() {
        None => Ok(ReceiverKind::None),
        Some(receiver) => {
            if receiver.colon_token.is_some() {
                return Err(Error::new_spanned(
                    receiver,
                    "#[uactor::service] does not support explicit `self: Type` receivers",
                ));
            }
            match (receiver.reference.is_some(), receiver.mutability.is_some()) {
                (false, _) => Err(Error::new_spanned(
                    receiver,
                    "#[uactor::service] methods cannot take `self` by value",
                )),
                (true, false) => Ok(ReceiverKind::Ref),
                (true, true) => Ok(ReceiverKind::Mut),
            }
        }
    }
}

fn classify_params(sig: &Signature) -> Result<Vec<Param>> {
    let mut params = Vec::new();
    for arg in &sig.inputs {
        match arg {
            FnArg::Receiver(_) => {}
            FnArg::Typed(pat_type) => {
                let ident = match &*pat_type.pat {
                    Pat::Ident(pat_ident)
                        if pat_ident.by_ref.is_none() && pat_ident.subpat.is_none() =>
                    {
                        pat_ident.ident.clone()
                    }
                    other => {
                        return Err(Error::new_spanned(
                            other,
                            "#[uactor::service] parameters must be simple identifiers",
                        ));
                    }
                };

                if has_ident_attr(&pat_type.attrs, "inject") {
                    params.push(Param::Inject { ident });
                    continue;
                }

                let name = ident.to_string();
                if name == "ctx" || name == "context" {
                    params.push(Param::Ctx { ident });
                } else if name == "state" {
                    params.push(Param::State { ident });
                } else {
                    params.push(Param::Field {
                        ident,
                        ty: (*pat_type.ty).clone(),
                    });
                }
            }
        }
    }
    Ok(params)
}

fn is_fire_and_forget_unit(sig: &Signature) -> bool {
    match &sig.output {
        ReturnType::Default => true,
        ReturnType::Type(_, ty) => {
            if is_unit_type(ty) {
                return true;
            }
            result_ok_type(ty).is_some_and(is_unit_type)
        }
    }
}

fn is_plain_unit_return(sig: &Signature) -> bool {
    match &sig.output {
        ReturnType::Default => true,
        ReturnType::Type(_, ty) => is_unit_type(ty),
    }
}

fn is_handle_result(sig: &Signature) -> bool {
    match &sig.output {
        ReturnType::Type(_, ty) => type_last_ident(ty).is_some_and(|id| id == "HandleResult"),
        ReturnType::Default => false,
    }
}

fn is_unit_type(ty: &Type) -> bool {
    matches!(ty, Type::Tuple(t) if t.elems.is_empty())
}

fn result_ok_type(ty: &Type) -> Option<&Type> {
    let Type::Path(path) = ty else {
        return None;
    };
    let last = path.path.segments.last()?;
    if last.ident != "Result" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(args) = &last.arguments else {
        return None;
    };
    args.args.iter().find_map(|arg| match arg {
        syn::GenericArgument::Type(inner) => Some(inner),
        _ => None,
    })
}

fn type_last_ident(ty: &Type) -> Option<&Ident> {
    match ty {
        Type::Path(path) => path.path.segments.last().map(|seg| &seg.ident),
        _ => None,
    }
}

fn actor_ident(ty: &Type) -> Result<Ident> {
    let Type::Path(path) = ty else {
        return Err(Error::new_spanned(
            ty,
            "#[uactor::service] requires a simple actor type name",
        ));
    };
    if path.qself.is_some() || path.path.segments.len() != 1 {
        return Err(Error::new_spanned(
            ty,
            "#[uactor::service] does not support qualified or module-prefixed actor types",
        ));
    }
    let segment = &path.path.segments[0];
    if !matches!(segment.arguments, syn::PathArguments::None) {
        return Err(Error::new_spanned(
            ty,
            "#[uactor::service] does not support generic actor types",
        ));
    }
    Ok(segment.ident.clone())
}

fn to_pascal_case(input: &str) -> String {
    let mut result = String::new();
    let mut upper = true;
    for c in input.chars() {
        if c == '_' {
            upper = true;
        } else if upper {
            result.extend(c.to_uppercase());
            upper = false;
        } else {
            result.push(c);
        }
    }
    if result.is_empty() {
        input.to_owned()
    } else {
        result
    }
}

fn strip_helper_attrs(method: &mut ImplItemFn) {
    method.attrs.retain(|attr| !is_helper_attr(attr));
    for arg in &mut method.sig.inputs {
        if let FnArg::Typed(pat_type) = arg {
            pat_type.attrs.retain(|attr| !is_helper_attr(attr));
        }
    }
}

fn is_helper_attr(attr: &Attribute) -> bool {
    ident_attr_matches(attr, &["skip", "inject"])
}

fn has_ident_attr(attrs: &[Attribute], name: &str) -> bool {
    attrs.iter().any(|attr| ident_attr_matches(attr, &[name]))
}

fn ident_attr_matches(attr: &Attribute, names: &[&str]) -> bool {
    let segments: Vec<_> = attr.path().segments.iter().collect();
    match segments.len() {
        1 => names.iter().any(|name| segments[0].ident == *name),
        2 => segments[0].ident == "uactor" && names.iter().any(|name| segments[1].ident == *name),
        _ => false,
    }
}
