extern crate proc_macro;

#[macro_use]
extern crate syn;

use proc_macro2::{Span, TokenStream};
use quote::quote;

use syn::{
    parse_macro_input, visit::Visit, visit_mut::VisitMut, Block, Expr,
    ExprYield, Generics, Ident, ItemFn, Lifetime, LifetimeDef, Receiver,
    ReturnType, TypeBareFn, TypeImplTrait, TypeParam, TypeParamBound,
    TypeReference,
};

// A `.await` expression is transformed into,
//
// {
//     let mut pinned = { /* the input expression */ }
//
//     loop {
//         let polled = unsafe {
//             let pin = ::core::pin::Pin::new_unchecked(&mut pinned);
//             ::core::future::Future::poll(
//                 pin,
//                 _embrio_async_context_argument.get_context(),
//             )
//         };
//         if let ::core::task::Poll::Ready(x) = polled {
//             break x;
//         }
//         yield ::core::task::Poll::Pending;
//     }
// }
fn await_impl(input: &Expr) -> Expr {
    let arg = Ident::new("_embrio_async_context_argument", Span::call_site());
    let expr = quote!({
        let mut pinned = #input;
        loop {
            // Safety: We trust users to only call this from within an
            // async_block created generator, they are static generators so must
            // be immovable in memory, so creating a pinned reference into a
            // generator-local is safe. de-referencing the argument pointer is
            // safe for reasons explained in the embrio-async safety notes.
            let polled = unsafe {
                let pin = ::core::pin::Pin::new_unchecked(&mut pinned);
                ::core::future::Future::poll(pin, #arg.get_context())
            };
            if let ::core::task::Poll::Ready(x) = polled {
                break x;
            }
            yield ::core::task::Poll::Pending;
        }
    });
    syn::parse2(expr).unwrap()
}

// An `async` block is transformed into,
//
// {
//     unsafe {
//         ::embrio_async::make_future(<move> |mut _embrio_async_context_argument| {
//             static move || {
//                 if false {
//                     yield ::core::task::Poll::Pending
//                 }
//                 {
//                     /* the input block */
//                 }
//             }
//         })
//     }
// }
//
// `static` in `static move || { ... }` means that the generator may hold self-references across
// yield points.
fn async_block(expr_async: &mut syn::ExprAsync) -> Expr {
    let block = &mut expr_async.block;
    let mv = &expr_async.capture;
    syn::visit_mut::visit_block_mut(&mut ExpandAwait, block);
    let arg = Ident::new("_embrio_async_context_argument", Span::call_site());
    let tokens = quote!({
        // Safety: We trust users not to come here, see that argument name we
        // generated above and use that in their code to break our other safety
        // guarantees. Our use of it in await! is safe because of reasons
        // probably described in the embrio-async safety notes.
        unsafe {
            ::embrio_async::make_future(#mv |mut #arg| {
                static move || {
                    if false { yield ::core::task::Poll::Pending }
                    #block
                }
            })
        }
    });

    syn::parse2(tokens).unwrap()
}

// When an `async` block contains a `yield` keyword, then its transformed into a stream.
//
// {
//     unsafe {
//         ::embrio_async::make_stream(<move> |mut _embrio_async_context_argument| {
//             static move || {
//                 if false {
//                     yield ::core::task::Poll::Pending
//                 }
//                 {
//                     yield ::core::task::Poll::Ready({ ... });
//
//                     yield ::core::task::Poll::Ready({ ... });
//                 }
//             }
//         }
//     }
// }
fn async_stream_block(expr_async: &mut syn::ExprAsync) -> Expr {
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
        fn visit_expr_mut(&mut self, i: &mut Expr) {
            // Don't descend into closures
            if let Expr::Closure(_) = i {
                return;
            }
            syn::visit_mut::visit_expr_mut(self, i);
        }
    }

    let mut block = &mut expr_async.block;
    let capture = &expr_async.capture;
    syn::visit_mut::VisitMut::visit_block_mut(&mut ReplaceYields, &mut block);
    syn::visit_mut::VisitMut::visit_block_mut(&mut ExpandAwait, &mut block);
    let arg = Ident::new("_embrio_async_context_argument", Span::call_site());
    let stream = quote!({
        // Safety: We trust users not to come here, see that argument name we
        // generated above and use that in their code to break our other safety
        // guarantees. Our use of it in await! is safe because of reasons
        // probably described in the embrio-async safety notes.
        unsafe {
            ::embrio_async::make_stream(#capture |mut #arg| {
                static move || {
                    if false { yield ::core::task::Poll::Pending }
                    #block
                }
            })
        }
    });

    syn::parse2(stream).unwrap()
}

#[proc_macro_attribute]
pub fn embrio_async(
    attr: proc_macro::TokenStream,
    body: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    assert!(attr.is_empty(), "async_fn attribute takes no arguments");
    async_fn_impl(parse_macro_input!(body)).into()
}

fn async_fn_impl(mut item: ItemFn) -> TokenStream {
    if item.sig.asyncness.is_some() {
        item.sig.asyncness = None;
        syn::visit_mut::visit_item_fn_mut(
            &mut AsyncFnTransform::default(),
            &mut item,
        );
    }

    let mut pinned = ExpandPinned::default();
    syn::visit_mut::visit_block_mut(&mut pinned, &mut item.block);
    let pin_attr = if pinned.expanded {
        Some(quote!{ #[::ergo_pin::ergo_pin] })
    } else {
        None
    };

    syn::visit_mut::visit_block_mut(&mut AsyncBlockTransform, &mut item.block);

    quote! {
        #pin_attr
        #item
    }
}

struct ExpandAwait;

impl syn::visit_mut::VisitMut for ExpandAwait {
    fn visit_expr_mut(&mut self, node: &mut syn::Expr) {
        syn::visit_mut::visit_expr_mut(self, node);
        let base = match node {
            syn::Expr::Await(syn::ExprAwait { base, .. }) => &*base,
            _ => return,
        };

        *node = await_impl(base);
    }
}

#[derive(Default)]
struct ExpandPinned {
    expanded: bool,
}

impl syn::visit_mut::VisitMut for ExpandPinned {
    fn visit_expr_mut(&mut self, node: &mut syn::Expr) {
        syn::visit_mut::visit_expr_mut(self, node);
        let (attrs, mac, pin_trait) = match node {
            Expr::Macro(syn::ExprMacro { attrs, mac }) => if mac.path.is_ident("pinned") {
                (attrs.clone(), mac.clone(), quote!(PinnableFuture))
            } else if mac.path.is_ident("pinned_stream") {
                (attrs.clone(), mac.clone(), quote!(PinnableStream))
            } else {
                return
            },
            _ => return,
        };
        self.expanded = true;
        let tokens = mac.tokens;
        let tokens: Expr = parse_quote!(#tokens);
        *node = syn::parse_quote! {
            #(#attrs)*
            ::embrio_async::#pin_trait::pin(#tokens, pin!(::core::default::Default::default()))
        };
    }
}

#[derive(Default)]
struct AsyncBlockTransform;

impl VisitMut for AsyncBlockTransform {
    fn visit_expr_mut(&mut self, i: &mut Expr) {
        syn::visit_mut::visit_expr_mut(self, i);
        let fut = match i {
            Expr::Async(expr_async) => {
                if contains_yield(&expr_async.block) {
                    async_stream_block(expr_async)
                } else {
                    async_block(expr_async)
                }
            }
            _ => {
                return;
            }
        };

        *i = fut;
    }
}

fn contains_yield(block: &Block) -> bool {
    struct ContainsYield(bool);

    impl<'a> Visit<'a> for ContainsYield {
        fn visit_expr_yield(&mut self, _: &'a ExprYield) {
            self.0 = true;
        }

        fn visit_expr(&mut self, i: &'a Expr) {
            // Don't descend into closures
            if let Expr::Closure(_) = i {
                return;
            }
            syn::visit::visit_expr(self, i);
        }
    }

    let mut visitor = ContainsYield(false);
    syn::visit::visit_block(&mut visitor, block);
    visitor.0
}

#[derive(Default)]
struct AsyncFnTransform {
    original_lifetimes: Vec<Lifetime>,
}

fn future_lifetime() -> Lifetime {
    Lifetime::new("'future", Span::call_site())
}

// Transforms a function of form
//
// ```
// #[embiro_async]
// async fn name(...) -> ReturnType {
//    ...
// }
// ```
//
// into
//
// ```
// fn name(...) -> impl Future<Output = ReturnType + ...> {
//    ...
// }
// ```
impl VisitMut for AsyncFnTransform {
    fn visit_type_reference_mut(&mut self, i: &mut TypeReference) {
        if i.lifetime.is_none() {
            i.lifetime = future_lifetime().into();
        }
        self.visit_type_mut(&mut *i.elem);
    }
    fn visit_receiver_mut(&mut self, i: &mut Receiver) {
        match i {
            Receiver {
                reference: Some((_, lifetime)),
                ..
            } if lifetime.is_none() => *lifetime = future_lifetime().into(),
            _ => (),
        }
    }
    fn visit_type_bare_fn_mut(&mut self, _i: &mut TypeBareFn) {}
    fn visit_type_impl_trait_mut(&mut self, i: &mut TypeImplTrait) {
        for bound in i.bounds.iter_mut() {
            self.visit_type_param_bound_mut(bound);
        }
        i.bounds.push(TypeParamBound::Lifetime(future_lifetime()));
    }
    fn visit_type_param_mut(&mut self, i: &mut TypeParam) {
        if i.colon_token.is_none() {
            i.colon_token = Some(Token![:](Span::call_site()));
        }
        for bound in i.bounds.iter_mut() {
            self.visit_type_param_bound_mut(bound);
        }
        i.bounds.push(future_lifetime().into())
    }
    fn visit_lifetime_mut(&mut self, i: &mut Lifetime) {
        if i.ident == "_" {
            *i = future_lifetime();
        }
    }
    fn visit_lifetime_def_mut(&mut self, i: &mut LifetimeDef) {
        if i.colon_token.is_none() {
            i.colon_token = Some(Token![:](Span::call_site()));
        }
        i.bounds.push(future_lifetime());
    }
    fn visit_generics_mut(&mut self, i: &mut Generics) {
        self.original_lifetimes =
            i.lifetimes().map(|lt| lt.lifetime.clone()).collect();
        for param in i.params.iter_mut() {
            self.visit_generic_param_mut(param);
        }
        i.params
            .insert(0, LifetimeDef::new(future_lifetime()).into());
    }
    fn visit_block_mut(&mut self, i: &mut Block) {
        *i = syn::parse_quote!({ async move #i });
    }
    fn visit_return_type_mut(&mut self, i: &mut ReturnType) {
        let lifetimes = &self.original_lifetimes;
        *i = syn::parse2(match i {
            ReturnType::Default => quote! {
                -> impl ::core::future::Future<Output = ()> #(+ ::embrio_async::Captures<#lifetimes>)* + ::embrio_async::PinnableFuture<FutureOutput = ()> + 'future
            },
            ReturnType::Type(_, ty) => quote! {
                -> impl ::core::future::Future<Output = #ty> #(+ ::embrio_async::Captures<#lifetimes>)* + ::embrio_async::PinnableFuture<FutureOutput = #ty> + 'future
            },
        }).unwrap();
    }
}
