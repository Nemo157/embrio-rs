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
            // Safety: Trust me
            let polled = unsafe {
                let pin = ::core::pin::Pin::new_unchecked(&mut pinned);
                ::core::future::Future::poll(pin, &**#arg)
            };
            if let ::core::task::Poll::Ready(x) = polled {
                break x;
            }
            yield
        }
    }).into()
}

#[proc_macro]
pub fn async_block(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: TokenStream = input.into();
    let arg = Ident::new("_embrio_async_lw_argument", Span::call_site());
    quote!({
        ::embrio_async::make_future(move |#arg| {
            static move || {
                if false { yield }
                #input
            }
        })
    }).into()
}
