extern crate proc_macro;

#[macro_use]
extern crate syn;

use proc_macro2::{Span, TokenStream};
use quote::quote;

use syn::{
    parse_macro_input, visit::Visit, visit_mut::VisitMut, Block, Expr,
    ExprYield, Generics, Ident, ItemFn, Lifetime, LifetimeDef, Pat, PatType,
    Receiver, ReturnType, TypeBareFn, TypeImplTrait, TypeParam, TypeParamBound,
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
fn await_impl(context_arg: &Ident, input: &Expr) -> Expr {
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
                ::core::future::Future::poll(pin, #context_arg.get_context())
            };
            if let ::core::task::Poll::Ready(x) = polled {
                break x;
            }
            #context_arg = yield ::core::task::Poll::Pending;
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
    let context_arg =
        Ident::new("_embrio_async_context_argument", Span::call_site());
    let block = &mut expr_async.block;
    let mv = &expr_async.capture;
    syn::visit_mut::visit_block_mut(&mut ExpandAwait(&context_arg), block);
    let stmts = &block.stmts;
    let tokens = quote!({
        // Safety: We trust users not to come here, see that argument name we
        // generated above and use that in their code to break our other safety
        // guarantees. Our use of it in await! is safe because of reasons
        // probably described in the embrio-async safety notes.
        unsafe {
            ::embrio_async::make_future(static #mv |mut #context_arg: ::embrio_async::UnsafeContextRef| {
                if false { #context_arg = yield ::core::task::Poll::Pending; }
                #(#stmts)*
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
                syn::parse_quote!(::core::task::Poll::Ready(#expr)),
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

    let context_arg =
        Ident::new("_embrio_async_context_argument", Span::call_site());
    let mut block = &mut expr_async.block;
    let capture = &expr_async.capture;
    syn::visit_mut::VisitMut::visit_block_mut(&mut ReplaceYields, &mut block);
    syn::visit_mut::VisitMut::visit_block_mut(
        &mut ExpandAwait(&context_arg),
        &mut block,
    );
    let stream = quote!({
        // Safety: We trust users not to come here, see that argument name we
        // generated above and use that in their code to break our other safety
        // guarantees. Our use of it in await! is safe because of reasons
        // probably described in the embrio-async safety notes.
        unsafe {
            ::embrio_async::make_stream(static #capture |mut #context_arg: ::embrio_async::UnsafeContextRef| {
                if false { #context_arg = yield ::core::task::Poll::Pending; }
                #block
            })
        }
    });

    syn::parse2(stream).unwrap()
}

// When an `async` closure contains a `yield` keyword, then its transformed into a sink.
fn async_sink_block(closure: &mut syn::ExprClosure) -> Expr {
    struct ReplaceYieldsInSink<'a>(&'a Ident);

    impl syn::visit_mut::VisitMut for ReplaceYieldsInSink<'_> {
        fn visit_expr_mut(&mut self, i: &mut Expr) {
            let unit: Expr = syn::parse_quote!(());
            match i {
                Expr::Closure(_) => {
                    // Don't descend into closures
                }
                Expr::Yield(node) => {
                    assert!(
                        node.expr.is_none()
                            || node.expr.as_deref() == Some(&unit),
                        "Cannot yield values in a sink"
                    );
                    let context_arg = self.0;
                    *i = syn::parse_quote!({
                        let mut maybe_item = ::core::option::Option::None;
                        loop {
                            match maybe_item {
                                ::core::option::Option::Some(item) => break item,
                                ::core::option::Option::None => {
                                    #context_arg = match #context_arg {
                                        ::embrio_async::SinkContext::StartSend(item) => {
                                            maybe_item = ::core::option::Option::Some(::core::option::Option::Some(item));
                                            yield ::embrio_async::SinkResult::Accepted
                                        }
                                        ::embrio_async::SinkContext::Flush(cx) => {
                                            yield ::embrio_async::SinkResult::Idle
                                        }
                                        ::embrio_async::SinkContext::Close(cx) => {
                                            maybe_item = ::core::option::Option::Some(::core::option::Option::None);
                                            ::embrio_async::SinkContext::Close(cx)
                                        }
                                    }
                                }
                            }
                        }
                    });
                }
                i => {
                    syn::visit_mut::visit_expr_mut(self, i);
                }
            }
        }
    }

    let context_arg =
        Ident::new("_embrio_async_sink_context_argument", Span::call_site());
    let mut body = &mut closure.body;
    let capture = &closure.capture;
    assert!(
        closure.inputs.len() < 2,
        "a sink must take either 0 or 1 arguments"
    );
    // Pretty hacky, use the type of an unnamed input as the item type and the
    // return type as the error type
    let input_ty = match closure.inputs.first() {
        Some(pat) => match pat {
            Pat::Type(PatType { pat, ty, .. }) if &*pat == &syn::parse_quote!(_) => {
                ty.clone()
            }
            _ => panic!("a sink argument must be `_: T` where `T` is the input item type"),
        }
        None => syn::parse_quote!(_),
    };
    let error_ty = match &closure.output {
        ReturnType::Default => panic!("a sink return type representing the error must be given as inference doesn't work"),
        ReturnType::Type(_, ty) => ty.clone(),
    };
    syn::visit_mut::VisitMut::visit_expr_mut(
        &mut ReplaceYieldsInSink(&context_arg),
        &mut body,
    );
    syn::visit_mut::VisitMut::visit_expr_mut(
        &mut ExpandAwaitInSink(&context_arg),
        &mut body,
    );
    syn::parse_quote!({
        // Safety: We trust users not to come here, see that argument name we
        // generated above and use that in their code to break our other safety
        // guarantees. Our use of it in await! is safe because of reasons
        // probably described in the embrio-async safety notes.
        unsafe {
            ::embrio_async::make_sink(static #capture |mut #context_arg: ::embrio_async::SinkContext<#input_ty>| {
                if false { #context_arg = yield ::embrio_async::SinkResult::Idle; }
                ::core::result::Result::Ok::<_, #error_ty>(#body)
            })
        }
    })
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

    syn::visit_mut::visit_block_mut(&mut AsyncBlockTransform, &mut item.block);

    quote!(#item)
}

struct ExpandAwait<'a>(&'a Ident);

impl syn::visit_mut::VisitMut for ExpandAwait<'_> {
    fn visit_expr_mut(&mut self, node: &mut syn::Expr) {
        syn::visit_mut::visit_expr_mut(self, node);
        let base = match node {
            syn::Expr::Await(syn::ExprAwait { base, .. }) => &*base,
            _ => return,
        };

        *node = await_impl(self.0, base);
    }
}

struct ExpandAwaitInSink<'a>(&'a Ident);

impl syn::visit_mut::VisitMut for ExpandAwaitInSink<'_> {
    fn visit_expr_mut(&mut self, node: &mut syn::Expr) {
        syn::visit_mut::visit_expr_mut(self, node);

        let input = match node {
            syn::Expr::Await(syn::ExprAwait { base, .. }) => &*base,
            _ => return,
        };

        let context_arg = self.0;
        *node = syn::parse_quote!({
            enum MaybeDone<F, T> { NotDone(F), Done(::core::option::Option<T>) }
            let mut maybe_future = MaybeDone::NotDone(#input);
            loop {
                // Safety: We trust users to only call this from within an
                // async_block created generator, they are static generators so must
                // be immovable in memory, so creating a pinned reference into a
                // generator-local is safe. de-referencing the argument pointer is
                // safe for reasons explained in the embrio-async safety notes.
                match &mut maybe_future {
                    MaybeDone::NotDone(future) => {
                        #context_arg = match #context_arg {
                            ::embrio_async::SinkContext::StartSend(item) => {
                                yield ::embrio_async::SinkResult::NotReady
                            }
                            | ::embrio_async::SinkContext::Flush(mut cx) => {
                                let polled = unsafe {
                                    let pin = ::core::pin::Pin::new_unchecked(future);
                                    ::core::future::Future::poll(pin, cx.get_context())
                                };
                                if let ::core::task::Poll::Ready(x) = polled {
                                    maybe_future = MaybeDone::Done(::core::option::Option::Some(x));
                                }
                                ::embrio_async::SinkContext::Flush(cx)
                            }
                            | ::embrio_async::SinkContext::Close(mut cx) => {
                                let polled = unsafe {
                                    let pin = ::core::pin::Pin::new_unchecked(future);
                                    ::core::future::Future::poll(pin, cx.get_context())
                                };
                                if let ::core::task::Poll::Ready(x) = polled {
                                    maybe_future = MaybeDone::Done(::core::option::Option::Some(x));
                                }
                                ::embrio_async::SinkContext::Close(cx)
                            }
                        };
                    }
                    MaybeDone::Done(e) => {
                        break e.take().unwrap();
                    }
                }
            }
        });
    }
}

#[derive(Default)]
struct AsyncBlockTransform;

impl VisitMut for AsyncBlockTransform {
    fn visit_expr_mut(&mut self, i: &mut Expr) {
        syn::visit_mut::visit_expr_mut(self, i);

        let expr_contains_yield = contains_yield(i);
        match i {
            Expr::Async(expr_async) => {
                if expr_contains_yield {
                    *i = async_stream_block(expr_async);
                } else {
                    *i = async_block(expr_async);
                }
            }
            Expr::Closure(closure) => {
                if closure.asyncness.is_some() {
                    if contains_yield(&closure.body) {
                        *i = async_sink_block(closure);
                    } else {
                        panic!("async closures are unsupported: {:?}", closure)
                    }
                }
            }
            _ => (),
        }
    }
}

fn contains_yield(block: &Expr) -> bool {
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
    syn::visit::visit_expr(&mut visitor, block);
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
                -> impl ::core::future::Future<Output = ()> #(+ ::embrio_async::Captures<#lifetimes>)* + 'future
            },
            ReturnType::Type(_, ty) => quote! {
                -> impl ::core::future::Future<Output = #ty> #(+ ::embrio_async::Captures<#lifetimes>)* + 'future
            },
        }).unwrap();
    }
}
