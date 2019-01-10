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
            yield
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
                    if false { yield }
                    #input
                }
            })
        }
    })
    .into()
}
