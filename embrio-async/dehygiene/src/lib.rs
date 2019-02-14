#![feature(async_await)]
#![recursion_limit = "128"]

extern crate proc_macro;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

#[proc_macro]
pub fn await(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: TokenStream = input.into();
    let context_arg =
        Ident::new("_embrio_async_context_argument", Span::call_site());
    quote!({
        let mut pinned = #input;
        loop {
            // Safety: We trust users to only call this from within an
            // async_block created generator, they are static generators so must
            // be immovable in memory, so creating a pinned reference into a
            // generator-local is safe. de-referencing the argument pointer is
            // safe for reasons explained in the embrio-async safety notes.
            let context = unsafe { #context_arg.get_context() };
            if let ::core::option::Option::Some(context) = context {
                let pin = unsafe { ::core::pin::Pin::new_unchecked(&mut pinned) };
                let polled = ::core::future::Future::poll(pin, context);
                if let ::core::task::Poll::Ready(x) = polled {
                    break x;
                }
            }
            yield ::core::task::Poll::Pending;
        }
    })
    .into()
}

#[proc_macro]
pub fn await_input(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    assert_eq!(input.to_string(), "");
    let item_arg =
        Ident::new("_embrio_async_sink_item_argument", Span::call_site());
    quote!({
        loop {
            let item = unsafe { #item_arg.get_item() };
            if let ::core::task::Poll::Ready(item) = item {
                break item;
            }
            yield ::core::task::Poll::Pending;
        }
    })
    .into()
}

#[proc_macro]
pub fn async_block(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: TokenStream = input.into();
    let context_arg =
        Ident::new("_embrio_async_context_argument", Span::call_site());
    quote!({
        // Safety: We trust users not to come here, see that argument name we
        // generated above and use that in their code to break our other safety
        // guarantees. Our use of it in await! is safe because of reasons
        // probably described in the embrio-async safety notes.
        unsafe {
            ::embrio_async::make_future(|#context_arg: ::embrio_async::UnsafeContextRef| {
                static || {
                    let mut #context_arg = #context_arg;
                    if false { yield ::core::task::Poll::Pending }
                    #input
                }
            })
        }
    })
    .into()
}

#[proc_macro]
pub fn async_stream_block(
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    struct ReplaceYields;

    impl syn::visit_mut::VisitMut for ReplaceYields {
        fn visit_expr_yield_mut(&mut self, node: &mut syn::ExprYield) {
            syn::visit_mut::visit_expr_yield_mut(self, node);
            let expr = node
                .expr
                .take()
                .unwrap_or_else(|| syn::parse_str("()").unwrap());
            node.expr = Some(Box::new(
                syn::parse2(quote!(::core::task::Poll::Ready(#expr))).unwrap(),
            ));
        }
    }

    let input: TokenStream = input.into();
    let mut input: syn::Block = syn::parse2(quote!({ #input })).unwrap();
    syn::visit_mut::VisitMut::visit_block_mut(&mut ReplaceYields, &mut input);
    let context_arg =
        Ident::new("_embrio_async_context_argument", Span::call_site());
    quote!({
        // Safety: We trust users not to come here, see that argument name we
        // generated above and use that in their code to break our other safety
        // guarantees. Our use of it in await! is safe because of reasons
        // probably described in the embrio-async safety notes.
        unsafe {
            ::embrio_async::make_stream(|#context_arg: ::embrio_async::UnsafeContextRef| {
                static || {
                    let mut #context_arg = #context_arg;
                    if false { yield ::core::task::Poll::Pending }
                    #input
                }
            })
        }
    })
    .into()
}

#[proc_macro]
pub fn async_sink_block(
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input: TokenStream = input.into();
    let context_arg =
        Ident::new("_embrio_async_context_argument", Span::call_site());
    let item_arg =
        Ident::new("_embrio_async_sink_item_argument", Span::call_site());
    quote!({
        // Safety: We trust users not to come here, see that argument name we
        // generated above and use that in their code to break our other safety
        // guarantees. Our use of it in await! is safe because of reasons
        // probably described in the embrio-async safety notes.
        unsafe {
            ::embrio_async::make_sink(|#context_arg: ::embrio_async::UnsafeContextRef, #item_arg: ::embrio_async::UnsafeItemRef<_>| {
                static || {
                    let mut #context_arg = #context_arg;
                    let #item_arg = #item_arg;
                    if false { yield ::core::task::Poll::Pending }
                    #input
                }
            })
        }
    })
    .into()
}
