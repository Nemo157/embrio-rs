#![feature(async_await)]

extern crate proc_macro;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

#[proc_macro]
pub fn await(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: TokenStream = input.into();
    let arg = Ident::new("_embrio_async_lw_argument", Span::call_site());
    quote!({
        let mut pinned = #input;
        loop {
            // Safety: We trust users to only call this from within an
            // async_block created generator, they are static generators so must
            // be immovable in memory, so creating a pinned reference into a
            // generator-local is safe. de-referencing the argument pointer is
            // safe for reasons explained in the embrio-async safety notes.
            let polled = unsafe {
                let pin = ::core::pin::Pin::new_unchecked(&mut pinned);
                ::core::future::Future::poll(pin, #arg.get_waker())
            };
            if let ::core::task::Poll::Ready(x) = polled {
                break x;
            }
            yield ::core::option::Option::None;
        }
    })
    .into()
}

#[proc_macro]
pub fn async_block(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: TokenStream = input.into();
    let arg = Ident::new("_embrio_async_lw_argument", Span::call_site());
    quote!({
        // Safety: We trust users not to come here, see that argument name we
        // generated above and use that in their code to break our other safety
        // guarantees. Our use of it in await! is safe because of reasons
        // probably described in the embrio-async safety notes.
        unsafe {
            ::embrio_async::make_future(move |#arg| {
                static move || {
                    if false { yield ::core::option::Option::None }
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
                syn::parse2(quote!(::core::option::Option::Some(#expr)))
                    .unwrap(),
            ));
        }
    }

    let input: TokenStream = input.into();
    let mut input: syn::Block = syn::parse2(quote!({ #input })).unwrap();
    syn::visit_mut::VisitMut::visit_block_mut(&mut ReplaceYields, &mut input);
    let arg = Ident::new("_embrio_async_lw_argument", Span::call_site());
    quote!({
        // Safety: We trust users not to come here, see that argument name we
        // generated above and use that in their code to break our other safety
        // guarantees. Our use of it in await! is safe because of reasons
        // probably described in the embrio-async safety notes.
        unsafe {
            ::embrio_async::make_stream(move |#arg| {
                static move || {
                    if false { yield ::core::option::Option::None }
                    #input
                }
            })
        }
    })
    .into()
}
