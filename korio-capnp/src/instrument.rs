use capnp::capability::{Promise, RemotePromise, Request};
use capnp::private::capability::{ClientHook, PipelineHook, RequestHook};
use korio::instrument::WithSubscriber;
use korio::{Dispatch, KorioInstrument, Span, dispatcher};

#[derive(Clone, Debug)]
#[must_use = "hooks do nothing unless you use them"]
pub struct WithDispatch<T> {
    inner: T,
    dispatcher: Dispatch,
}

impl<T> WithDispatch<T> {
    pub fn new(inner: T, subscriber: impl Into<Dispatch>) -> Self {
        Self {
            inner,
            dispatcher: subscriber.into(),
        }
    }

    pub fn wrap_current(inner: T) -> Self {
        Self::new(inner, Dispatch::default())
    }
}

impl ClientHook for WithDispatch<Box<dyn ClientHook>> {
    fn add_ref(&self) -> Box<dyn ClientHook> {
        let _default = dispatcher::set_default(&self.dispatcher);
        Box::new(Self {
            dispatcher: self.dispatcher.clone(),
            inner: self.inner.add_ref(),
        })
    }

    fn new_call(
        &self,
        interface_id: u64,
        method_id: u16,
        size_hint: Option<capnp::MessageSize>,
    ) -> Request<capnp::any_pointer::Owned, capnp::any_pointer::Owned> {
        let _default = dispatcher::set_default(&self.dispatcher);
        let req = self.inner.new_call(interface_id, method_id, size_hint);
        Request::new(Box::new(WithDispatch::wrap_current(req.hook)))
    }

    fn call(
        &self,
        interface_id: u64,
        method_id: u16,
        params: Box<dyn capnp::private::capability::ParamsHook>,
        results: Box<dyn capnp::private::capability::ResultsHook>,
    ) -> capnp::capability::Promise<(), capnp::Error> {
        let _default = dispatcher::set_default(&self.dispatcher);
        let fut = self.inner.call(interface_id, method_id, params, results);
        Promise::from_future(fut.with_current_subscriber())
    }

    fn get_brand(&self) -> usize {
        let _default = dispatcher::set_default(&self.dispatcher);
        self.inner.get_brand()
    }

    fn get_ptr(&self) -> usize {
        let _default = dispatcher::set_default(&self.dispatcher);
        self.inner.get_ptr()
    }

    fn get_resolved(&self) -> Option<Box<dyn ClientHook>> {
        let _default = dispatcher::set_default(&self.dispatcher);
        if let Some(resolved) = self.inner.get_resolved() {
            Some(Box::new(WithDispatch::wrap_current(resolved)))
        } else {
            None
        }
    }

    fn when_more_resolved(
        &self,
    ) -> Option<capnp::capability::Promise<Box<dyn ClientHook>, capnp::Error>> {
        let _default = dispatcher::set_default(&self.dispatcher);
        self.inner.when_more_resolved().map(|fut| {
            Promise::from_future(
                async move {
                    let hook = fut.await?;
                    Ok(Box::new(WithDispatch::wrap_current(hook)) as Box<dyn ClientHook>)
                }
                .with_current_subscriber(),
            )
        })
    }

    fn when_resolved(&self) -> capnp::capability::Promise<(), capnp::Error> {
        let _default = dispatcher::set_default(&self.dispatcher);
        Promise::from_future(self.inner.when_resolved().with_current_subscriber())
    }
}

impl RequestHook for WithDispatch<Box<dyn RequestHook>> {
    fn get(&mut self) -> capnp::any_pointer::Builder<'_> {
        let _default = dispatcher::set_default(&self.dispatcher);
        self.inner.get()
    }

    fn get_brand(&self) -> usize {
        let _default = dispatcher::set_default(&self.dispatcher);
        self.inner.get_brand()
    }

    fn send(self: Box<Self>) -> RemotePromise<capnp::any_pointer::Owned> {
        let _default = dispatcher::set_default(&self.dispatcher);
        let response = self.inner.send();
        RemotePromise {
            promise: Promise::from_future(response.promise.with_current_subscriber()),
            pipeline: capnp::any_pointer::Pipeline::new(Box::new(WithDispatch::wrap_current(
                response.pipeline.hook,
            ))),
        }
    }

    fn send_streaming(self: Box<Self>) -> Promise<(), capnp::Error> {
        let _default = dispatcher::set_default(&self.dispatcher);
        Promise::from_future(self.inner.send_streaming().with_current_subscriber())
    }

    fn tail_send(
        self: Box<Self>,
    ) -> Option<(
        u32,
        capnp::capability::Promise<(), capnp::Error>,
        Box<dyn capnp::private::capability::PipelineHook>,
    )> {
        let _default = dispatcher::set_default(&self.dispatcher);
        if let Some((idx, fut, hook)) = self.inner.tail_send() {
            Some((
                idx,
                Promise::from_future(fut.with_current_subscriber()),
                Box::new(WithDispatch::wrap_current(hook)),
            ))
        } else {
            None
        }
    }
}

impl PipelineHook for WithDispatch<Box<dyn PipelineHook>> {
    fn add_ref(&self) -> Box<dyn PipelineHook> {
        let _default = dispatcher::set_default(&self.dispatcher);
        Box::new(Self {
            dispatcher: self.dispatcher.clone(),
            inner: self.inner.add_ref(),
        })
    }

    fn get_pipelined_cap(
        &self,
        ops: &[capnp::private::capability::PipelineOp],
    ) -> Box<dyn ClientHook> {
        let _default = dispatcher::set_default(&self.dispatcher);
        let hook = self.inner.get_pipelined_cap(ops);
        Box::new(WithDispatch::wrap_current(hook))
    }
}

#[derive(Clone, Debug)]
#[must_use = "hooks do nothing unless you use them"]
pub struct WithSpan<T> {
    inner: T,
    span: Span,
}

impl<T> WithSpan<T> {
    pub fn new(inner: T, span: Span) -> Self {
        Self { inner, span }
    }

    pub fn wrap_current(inner: T) -> Self {
        Self::new(inner, Span::current())
    }
}

impl ClientHook for WithSpan<Box<dyn ClientHook>> {
    fn add_ref(&self) -> Box<dyn ClientHook> {
        let _default = self.span.enter();
        Box::new(Self {
            span: self.span.clone(),
            inner: self.inner.add_ref(),
        })
    }

    fn new_call(
        &self,
        interface_id: u64,
        method_id: u16,
        size_hint: Option<capnp::MessageSize>,
    ) -> Request<capnp::any_pointer::Owned, capnp::any_pointer::Owned> {
        let _default = self.span.enter();
        let req = self.inner.new_call(interface_id, method_id, size_hint);
        Request::new(Box::new(WithSpan::wrap_current(req.hook)))
    }

    fn call(
        &self,
        interface_id: u64,
        method_id: u16,
        params: Box<dyn capnp::private::capability::ParamsHook>,
        results: Box<dyn capnp::private::capability::ResultsHook>,
    ) -> capnp::capability::Promise<(), capnp::Error> {
        let _default = self.span.enter();
        let fut = self.inner.call(interface_id, method_id, params, results);
        Promise::from_future(fut.in_current_korio_span())
    }

    fn get_brand(&self) -> usize {
        let _default = self.span.enter();
        self.inner.get_brand()
    }

    fn get_ptr(&self) -> usize {
        let _default = self.span.enter();
        self.inner.get_ptr()
    }

    fn get_resolved(&self) -> Option<Box<dyn ClientHook>> {
        let _default = self.span.enter();
        if let Some(resolved) = self.inner.get_resolved() {
            Some(Box::new(WithSpan::wrap_current(resolved)))
        } else {
            None
        }
    }

    fn when_more_resolved(
        &self,
    ) -> Option<capnp::capability::Promise<Box<dyn ClientHook>, capnp::Error>> {
        let _default = self.span.enter();
        self.inner.when_more_resolved().map(|fut| {
            Promise::from_future(
                async move {
                    let hook = fut.await?;
                    Ok(Box::new(WithSpan::wrap_current(hook)) as Box<dyn ClientHook>)
                }
                .in_current_korio_span(),
            )
        })
    }

    fn when_resolved(&self) -> capnp::capability::Promise<(), capnp::Error> {
        let _default = self.span.enter();
        Promise::from_future(self.inner.when_resolved().in_current_korio_span())
    }
}

impl RequestHook for WithSpan<Box<dyn RequestHook>> {
    fn get(&mut self) -> capnp::any_pointer::Builder<'_> {
        let _default = self.span.enter();
        self.inner.get()
    }

    fn get_brand(&self) -> usize {
        let _default = self.span.enter();
        self.inner.get_brand()
    }

    fn send(self: Box<Self>) -> RemotePromise<capnp::any_pointer::Owned> {
        let _default = self.span.enter();
        let response = self.inner.send();
        RemotePromise {
            promise: Promise::from_future(response.promise.in_current_korio_span()),
            pipeline: capnp::any_pointer::Pipeline::new(Box::new(WithSpan::wrap_current(
                response.pipeline.hook,
            ))),
        }
    }

    fn send_streaming(self: Box<Self>) -> Promise<(), capnp::Error> {
        let _default = self.span.enter();
        Promise::from_future(self.inner.send_streaming().in_current_korio_span())
    }

    fn tail_send(
        self: Box<Self>,
    ) -> Option<(
        u32,
        capnp::capability::Promise<(), capnp::Error>,
        Box<dyn capnp::private::capability::PipelineHook>,
    )> {
        let _default = self.span.enter();
        if let Some((idx, fut, hook)) = self.inner.tail_send() {
            Some((
                idx,
                Promise::from_future(fut.in_current_korio_span()),
                Box::new(WithSpan::wrap_current(hook)),
            ))
        } else {
            None
        }
    }
}

impl PipelineHook for WithSpan<Box<dyn PipelineHook>> {
    fn add_ref(&self) -> Box<dyn PipelineHook> {
        let _default = self.span.enter();
        Box::new(Self {
            span: self.span.clone(),
            inner: self.inner.add_ref(),
        })
    }

    fn get_pipelined_cap(
        &self,
        ops: &[capnp::private::capability::PipelineOp],
    ) -> Box<dyn ClientHook> {
        let _default = self.span.enter();
        let hook = self.inner.get_pipelined_cap(ops);
        Box::new(WithSpan::wrap_current(hook))
    }
}
