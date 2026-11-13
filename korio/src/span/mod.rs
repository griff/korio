use std::{borrow::Cow, cmp, fmt, marker::PhantomData, mem, ops::Deref, time::SystemTime};

use crate::metadata::Severity;

mod builder;

pub use builder::RefSpanBuilder;
#[doc(inline)]
pub use korio_core::span::*;
use korio_core::{
    dispatcher::{self, Dispatch},
    metadata::{AsKey, AttributeSet, Metadata, Value},
};

pub(crate) enum Parent {
    Root,
    Current,
    Explicit(Id),
}

struct Inner {
    id: Id,
    subscriber: Dispatch,
}

impl Inner {
    /// Returns the span's ID.
    fn id(&self) -> Id {
        self.id.clone()
    }

    fn new(id: Id, subscriber: &Dispatch) -> Self {
        Self {
            id,
            subscriber: subscriber.clone(),
        }
    }
}

impl cmp::PartialEq for Inner {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl std::hash::Hash for Inner {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Clone for Inner {
    fn clone(&self) -> Self {
        Inner {
            id: self.subscriber.clone_span(&self.id),
            subscriber: self.subscriber.clone(),
        }
    }
}

#[derive(Clone)]
pub struct Span {
    inner: Option<Inner>,
}

impl Span {
    pub const fn none() -> Span {
        Span { inner: None }
    }

    /// Returns a handle to the span [considered by the `Subscriber`] to be the
    /// current span.
    ///
    /// If the subscriber indicates that it does not track the current span, or
    /// that the thread from which this function is called is not currently
    /// inside a span, the returned span will be disabled.
    ///
    /// [considered by the `Subscriber`]:
    ///     super::subscriber::Subscriber::current_span
    pub fn current() -> Span {
        dispatcher::get_default(|dispatch| {
            if let Some(ctx) = dispatch.current_span().into_inner() {
                let id = dispatch.clone_span(&ctx);
                Self {
                    inner: Some(Inner::new(id, dispatch)),
                }
            } else {
                Self::none()
            }
        })
    }

    pub fn build<'n, 't, N, T>(
        target: T,
        name: N,
        level: Severity,
    ) -> RefSpanBuilder<'n, 't, 'static, &'static dyn AttributeSet>
    where
        N: Into<Cow<'n, str>>,
        T: Into<Cow<'t, str>>,
    {
        RefSpanBuilder::new(target, name, level)
    }

    pub fn build_child_of<'n, 't, N, T>(
        parent: impl Into<Option<Id>>,
        target: T,
        name: N,
        level: Severity,
    ) -> RefSpanBuilder<'n, 't, 'static, &'static dyn AttributeSet>
    where
        N: Into<Cow<'n, str>>,
        T: Into<Cow<'t, str>>,
    {
        RefSpanBuilder::child_of(parent, target, name, level)
    }

    pub fn from_upstream(id: &Id) -> Span {
        dispatcher::get_default(|dispatch| Span::from_upstream_with_dispatch(id, dispatch))
    }

    pub fn from_downstream(span: &dyn SpanData) -> Span {
        dispatcher::get_default(|dispatch| Span::from_downstream_with_dispatch(span, dispatch))
    }

    pub fn from_data(span: &dyn NewSpanData) -> Span {
        dispatcher::get_default(|dispatch| Span::from_data_with_dispatch(span, dispatch))
    }

    #[inline]
    pub(crate) fn from_data_with_dispatch(span: &dyn NewSpanData, dispatch: &Dispatch) -> Span {
        let id = dispatch.new_span(span);
        Span {
            inner: Some(Inner::new(id, dispatch)),
        }
    }

    #[inline]
    pub(crate) fn from_downstream_with_dispatch(span: &dyn SpanData, dispatch: &Dispatch) -> Span {
        dispatch.downstream_span(span);

        let inner = Some(Inner::new(span.id().clone(), dispatch));
        Span { inner }
    }

    #[inline]
    pub(crate) fn from_upstream_with_dispatch(ctx: &Id, dispatch: &Dispatch) -> Span {
        dispatch.upstream_span(ctx);
        Span {
            inner: Some(Inner::new(ctx.clone(), dispatch)),
        }
    }

    /// Enters this span, returning a guard that will exit the span when dropped.
    ///
    /// If this span is enabled by the current subscriber, then this function will
    /// call [`Subscriber::enter`] with the span's [`Id`], and dropping the guard
    /// will call [`Subscriber::exit`]. If the span is disabled, this does
    /// nothing.
    ///
    /// # In Asynchronous Code
    ///
    /// **Warning**: in asynchronous code that uses [async/await syntax][syntax],
    /// `Span::enter` should be used very carefully or avoided entirely. Holding
    /// the drop guard returned by `Span::enter` across `.await` points will
    /// result in incorrect traces. For example,
    ///
    /// ```
    /// # use korio::korio_info_span;
    /// # async fn some_other_async_function() {}
    /// async fn my_async_function() {
    ///     let span = korio_info_span!("my_async_function");
    ///
    ///     // WARNING: This span will remain entered until this
    ///     // guard is dropped...
    ///     let _enter = span.enter();
    ///     // ...but the `await` keyword may yield, causing the
    ///     // runtime to switch to another task, while remaining in
    ///     // this span!
    ///     some_other_async_function().await
    ///
    ///     // ...
    /// }
    /// ```
    ///
    /// The drop guard returned by `Span::enter` exits the span when it is
    /// dropped. When an async function or async block yields at an `.await`
    /// point, the current scope is _exited_, but values in that scope are
    /// **not** dropped (because the async block will eventually resume
    /// execution from that await point). This means that _another_ task will
    /// begin executing while _remaining_ in the entered span. This results in
    /// an incorrect trace.
    ///
    /// Instead of using `Span::enter` in asynchronous code, prefer the
    /// following:
    ///
    /// * To enter a span for a synchronous section of code within an async
    ///   block or function, prefer [`Span::in_scope`]. Since `in_scope` takes a
    ///   synchronous closure and exits the span when the closure returns, the
    ///   span will always be exited before the next await point. For example:
    ///   ```
    ///   # use korio::korio_info_span;
    ///   # async fn some_other_async_function(_: ()) {}
    ///   async fn my_async_function() {
    ///       let span = korio_info_span!("my_async_function");
    ///
    ///       let some_value = span.in_scope(|| {
    ///           // run some synchronous code inside the span...
    ///       });
    ///
    ///       // This is okay! The span has already been exited before we reach
    ///       // the await point.
    ///       some_other_async_function(some_value).await;
    ///
    ///       // ...
    ///   }
    ///   ```
    /// * For instrumenting asynchronous code, `tracing` provides the
    ///   [`Future::instrument` combinator][instrument] for
    ///   attaching a span to a future (async function or block). This will
    ///   enter the span _every_ time the future is polled, and exit it whenever
    ///   the future yields.
    ///
    ///   `Instrument` can be used with an async block inside an async function:
    ///   ```ignore
    ///   # use korio::korio_info_span;
    ///   use korio::Instrument;
    ///
    ///   # async fn some_other_async_function() {}
    ///   async fn my_async_function() {
    ///       let span = korio_info_span!("my_async_function");
    ///       async move {
    ///          // This is correct! If we yield here, the span will be exited,
    ///          // and re-entered when we resume.
    ///          some_other_async_function().await;
    ///
    ///          //more asynchronous code inside the span...
    ///
    ///       }
    ///         // instrument the async block with the span...
    ///         .instrument(span)
    ///         // ...and await it.
    ///         .await
    ///   }
    ///   ```
    ///
    ///   It can also be used to instrument calls to async functions at the
    ///   callsite:
    ///   ```ignore
    ///   # use korio::korio_debug_span;
    ///   use korio::Instrument;
    ///
    ///   # async fn some_other_async_function() {}
    ///   async fn my_async_function() {
    ///       let some_value = some_other_async_function()
    ///          .instrument(korio_debug_span!("some_other_async_function"))
    ///          .await;
    ///
    ///       // ...
    ///   }
    ///   ```
    ///
    /// * The [`#[instrument]` attribute macro][attr] can automatically generate
    ///   correct code when used on an async function:
    ///
    ///   ```ignore
    ///   # async fn some_other_async_function() {}
    ///   #[korio::instrument(level = "info")]
    ///   async fn my_async_function() {
    ///
    ///       // This is correct! If we yield here, the span will be exited,
    ///       // and re-entered when we resume.
    ///       some_other_async_function().await;
    ///
    ///       // ...
    ///
    ///   }
    ///   ```
    ///
    /// [syntax]: https://rust-lang.github.io/async-book/01_getting_started/04_async_await_primer.html
    /// [`Span::in_scope`]: Span::in_scope()
    /// [instrument]: crate::Instrument
    /// [attr]: macro@crate::instrument
    ///
    /// # Examples
    ///
    /// ```
    /// # use korio::{korio_span, Severity};
    /// let span = korio_span!(Severity::Info, "my_span");
    /// let guard = span.enter();
    ///
    /// // code here is within the span
    ///
    /// drop(guard);
    ///
    /// // code here is no longer within the span
    ///
    /// ```
    ///
    /// Guards need not be explicitly dropped:
    ///
    /// ```
    /// # use korio::korio_trace_span;
    /// fn my_function() -> String {
    ///     // enter a span for the duration of this function.
    ///     let span = korio_trace_span!("my_function");
    ///     let _enter = span.enter();
    ///
    ///     // anything happening in functions we call is still inside the span...
    ///     my_other_function();
    ///
    ///     // returning from the function drops the guard, exiting the span.
    ///     return "Hello world".to_owned();
    /// }
    ///
    /// fn my_other_function() {
    ///     // ...
    /// }
    /// ```
    ///
    /// Sub-scopes may be created to limit the duration for which the span is
    /// entered:
    ///
    /// ```
    /// # use korio::{korio_info, korio_info_span};
    /// let span = korio_info_span!("my_great_span");
    ///
    /// {
    ///     let _enter = span.enter();
    ///
    ///     // this event occurs inside the span.
    ///     korio_info!("i'm in the span!");
    ///
    ///     // exiting the scope drops the guard, exiting the span.
    /// }
    ///
    /// // this event is not inside the span.
    /// korio_info!("i'm outside the span!")
    /// ```
    ///
    /// [`Subscriber::enter`]: super::subscriber::Subscriber::enter()
    /// [`Subscriber::exit`]: super::subscriber::Subscriber::exit()
    /// [`Id`]: super::Id
    #[inline(always)]
    pub fn enter(&self) -> Entered<'_> {
        self.do_enter();
        Entered { span: self }
    }

    /// Enters this span, consuming it and returning a [guard][`EnteredSpan`]
    /// that will exit the span when dropped.
    ///
    /// <pre class="compile_fail" style="white-space:normal;font:inherit;">
    ///     <strong>Warning</strong>: In asynchronous code that uses async/await syntax,
    ///     <code>Span::entered</code> may produce incorrect traces if the returned drop
    ///     guard is held across an await point. See <a href="#in-asynchronous-code">the
    ///     <code>Span::enter</code> documentation</a> for details.
    /// </pre>
    ///
    ///
    /// If this span is enabled by the current subscriber, then this function will
    /// call [`Subscriber::enter`] with the span's [`Id`], and dropping the guard
    /// will call [`Subscriber::exit`]. If the span is disabled, this does
    /// nothing.
    ///
    /// This is similar to the [`Span::enter`] method, except that it moves the
    /// span by value into the returned guard, rather than borrowing it.
    /// Therefore, this method can be used to create and enter a span in a
    /// single expression, without requiring a `let`-binding. For example:
    ///
    /// ```
    /// # use korio::korio_info_span;
    /// let _span = korio_info_span!("something_interesting").entered();
    /// ```
    /// rather than:
    /// ```
    /// # use korio::korio_info_span;
    /// let span = korio_info_span!("something_interesting");
    /// let _e = span.enter();
    /// ```
    ///
    /// Furthermore, `entered` may be used when the span must be stored in some
    /// other struct or be passed to a function while remaining entered.
    ///
    /// <pre class="ignore" style="white-space:normal;font:inherit;">
    ///     <strong>Note</strong>: The returned <a href="../struct.EnteredSpan.html">
    ///     <code>EnteredSpan</code></a> guard does not implement <code>Send</code>.
    ///     Dropping the guard will exit <em>this</em> span, and if the guard is sent
    ///     to another thread and dropped there, that thread may never have entered
    ///     this span. Thus, <code>EnteredSpan</code>s should not be sent between threads.
    /// </pre>
    ///
    /// [syntax]: https://rust-lang.github.io/async-book/01_getting_started/04_async_await_primer.html
    ///
    /// # Examples
    ///
    /// The returned guard can be [explicitly exited][EnteredSpan::exit],
    /// returning the un-entered span:
    ///
    /// ```
    /// # use korio::{Severity, korio_span};
    /// let span = korio_span!(Severity::Info, "doing_something").entered();
    ///
    /// // code here is within the span
    ///
    /// // explicitly exit the span, returning it
    /// let span = span.exit();
    ///
    /// // code here is no longer within the span
    ///
    /// // enter the span again
    /// let span = span.entered();
    ///
    /// // now we are inside the span once again
    /// ```
    ///
    /// Guards need not be explicitly dropped:
    ///
    /// ```
    /// # use korio::korio_trace_span;
    /// fn my_function() -> String {
    ///     // enter a span for the duration of this function.
    ///     let span = korio_trace_span!("my_function").entered();
    ///
    ///     // anything happening in functions we call is still inside the span...
    ///     my_other_function();
    ///
    ///     // returning from the function drops the guard, exiting the span.
    ///     return "Hello world".to_owned();
    /// }
    ///
    /// fn my_other_function() {
    ///     // ...
    /// }
    /// ```
    ///
    /// Since the [`EnteredSpan`] guard can dereference to the [`Span`] itself,
    /// the span may still be accessed while entered. For example:
    ///
    /// ```rust
    /// # use korio::korio_info_span;
    ///
    /// // create the span, and enter it.
    /// let span = korio_info_span!("my_span").entered();
    ///
    /// // we can still record a value for the field while the span is entered.
    /// span.record("some_field", &"hello world!");
    /// ```
    ///
    /// [`Subscriber::enter`]: super::subscriber::Subscriber::enter()
    /// [`Subscriber::exit`]: super::subscriber::Subscriber::exit()
    /// [`Id`]: super::Id
    #[inline(always)]
    pub fn entered(self) -> EnteredSpan {
        self.do_enter();
        EnteredSpan {
            span: self,
            _not_send: PhantomData,
        }
    }

    /// Returns this span, if it was [enabled] by the current [`Subscriber`], or
    /// the [current span] (whose lexical distance may be further than expected),
    ///  if this span [is disabled].
    ///
    /// This method can be useful when propagating spans to spawned threads or
    /// [async tasks]. Consider the following:
    ///
    /// ```
    /// let _parent_span = korio::korio_info_span!("parent").entered();
    ///
    /// // ...
    ///
    /// let child_span = korio::korio_debug_span!("child");
    ///
    /// std::thread::spawn(move || {
    ///     let _entered = child_span.entered();
    ///
    ///     korio::korio_info!("spawned a thread!");
    ///
    ///     // ...
    /// });
    /// ```
    ///
    /// If the current [`Subscriber`] enables the [`DEBUG`] level, then both
    /// the "parent" and "child" spans will be enabled. Thus, when the "spawned
    /// a thread!" event occurs, it will be inside of the "child" span. Because
    /// "parent" is the parent of "child", the event will _also_ be inside of
    /// "parent".
    ///
    /// However, if the [`Subscriber`] only enables the [`INFO`] level, the "child"
    /// span will be disabled. When the thread is spawned, the
    /// `child_span.entered()` call will do nothing, since "child" is not
    /// enabled. In this case, the "spawned a thread!" event occurs outside of
    /// *any* span, since the "child" span was responsible for propagating its
    /// parent to the spawned thread.
    ///
    /// If this is not the desired behavior, `Span::or_current` can be used to
    /// ensure that the "parent" span is propagated in both cases, either as a
    /// parent of "child" _or_ directly. For example:
    ///
    /// ```
    /// let _parent_span = korio::korio_info_span!("parent").entered();
    ///
    /// // ...
    ///
    /// // If DEBUG is enabled, then "child" will be enabled, and `or_current`
    /// // returns "child". Otherwise, if DEBUG is not enabled, "child" will be
    /// // disabled, and `or_current` returns "parent".
    /// let child_span = korio::korio_debug_span!("child").or_current();
    ///
    /// std::thread::spawn(move || {
    ///     let _entered = child_span.entered();
    ///
    ///     korio::korio_info!("spawned a thread!");
    ///
    ///     // ...
    /// });
    /// ```
    ///
    /// When spawning [asynchronous tasks][async tasks], `Span::or_current` can
    /// be used similarly, in combination with [`korio_instrument`]:
    ///
    /// ```
    /// use korio::KorioInstrument;
    /// # // lol
    /// # mod tokio {
    /// #     pub(super) fn spawn(_: impl std::future::Future) {}
    /// # }
    ///
    /// let _parent_span = korio::korio_info_span!("parent").entered();
    ///
    /// // ...
    ///
    /// let child_span = korio::korio_debug_span!("child");
    ///
    /// tokio::spawn(
    ///     async {
    ///         korio::korio_info!("spawned a task!");
    ///
    ///         // ...
    ///
    ///     }.korio_instrument(child_span.or_current())
    /// );
    /// ```
    ///
    /// In general, `or_current` should be preferred over nesting an
    /// [`korio_instrument`]  call inside of an [`in_current_korio_span`] call, as using
    /// `or_current` will be more efficient.
    ///
    /// ```
    /// use korio::KorioInstrument;
    /// # // lol
    /// # mod tokio {
    /// #     pub(super) fn spawn(_: impl std::future::Future) {}
    /// # }
    /// async fn my_async_fn() {
    ///     // ...
    /// }
    ///
    /// let _parent_span = korio::korio_info_span!("parent").entered();
    ///
    /// // Do this:
    /// tokio::spawn(
    ///     my_async_fn().korio_instrument(korio::korio_debug_span!("child").or_current())
    /// );
    ///
    /// // ...rather than this:
    /// tokio::spawn(
    ///     my_async_fn()
    ///         .korio_instrument(korio::korio_debug_span!("child"))
    ///         .in_current_korio_span()
    /// );
    /// ```
    ///
    /// [enabled]: crate::Subscriber::enabled
    /// [`Subscriber`]: crate::Subscriber
    /// [current span]: Span::current
    /// [is disabled]: Span::is_disabled
    /// [`INFO`]: crate::Level::INFO
    /// [`DEBUG`]: crate::Level::DEBUG
    /// [async tasks]: std::task
    /// [`korio_instrument`]: crate::instrument::KorioInstrument::korio_instrument
    /// [`in_current_korio_span`]: crate::instrument::KorioInstrument::in_current_korio_span
    pub fn or_current(self) -> Self {
        if self.is_disabled() {
            return Self::current();
        }
        self
    }

    #[inline(always)]
    fn do_enter(&self) {
        if let Some(inner) = self.inner.as_ref() {
            inner.subscriber.enter(&inner.id);
        }
    }

    // Called from [`Entered`] and [`EnteredSpan`] drops.
    //
    // Running this behaviour on drop rather than with an explicit function
    // call means that spans may still be exited when unwinding.
    #[inline(always)]
    fn do_exit(&self) {
        if let Some(inner) = self.inner.as_ref() {
            inner.subscriber.exit(&inner.id);
        }
    }

    /// Executes the given function in the context of this span.
    ///
    /// If this span is enabled, then this function enters the span, invokes `f`
    /// and then exits the span. If the span is disabled, `f` will still be
    /// invoked, but in the context of the currently-executing span (if there is
    /// one).
    ///
    /// Returns the result of evaluating `f`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use korio::{korio_trace, korio_span, Severity};
    /// let my_span = korio_span!(Severity::Trace, "my_span");
    ///
    /// my_span.in_scope(|| {
    ///     // this event occurs within the span.
    ///     korio_trace!("i'm in the span!");
    /// });
    ///
    /// // this event occurs outside the span.
    /// korio_trace!("i'm not in the span!");
    /// ```
    ///
    /// Calling a function and returning the result:
    /// ```
    /// # use korio::korio_info_span;
    /// fn hello_world() -> String {
    ///     "Hello world!".to_owned()
    /// }
    ///
    /// let span = korio_info_span!("hello_world");
    /// // the span will be entered for the duration of the call to
    /// // `hello_world`.
    /// let a_string = span.in_scope(hello_world);
    ///
    pub fn in_scope<F: FnOnce() -> T, T>(&self, f: F) -> T {
        let _enter = self.enter();
        f()
    }

    /// Returns `true` if this span was disabled by the subscriber and does not
    /// exist.
    #[inline]
    pub fn is_disabled(&self) -> bool {
        self.inner.is_none()
    }

    /// Returns this span's `Id`, if it is enabled.
    pub fn id(&self) -> Option<Id> {
        self.inner.as_ref().map(Inner::id)
    }

    pub fn start_with_timestamp(&self, start_time: SystemTime) {
        self.with_subscriber(|(id, dispatch)| {
            dispatch.start(id, start_time);
        });
    }

    pub fn start(&self) {
        self.start_with_timestamp(SystemTime::now());
    }

    pub fn end_with_timestamp(&self, end_time: SystemTime) {
        self.with_subscriber(|(id, dispatch)| {
            dispatch.end(id, end_time);
        });
    }

    pub fn end(&self) {
        self.end_with_timestamp(SystemTime::now());
    }

    #[inline(always)]
    pub fn start_lifecycle(&self) -> EndSpan<'_> {
        self.start();
        EndSpan { span: self }
    }

    #[inline(always)]
    pub fn lifecycle(self) -> EndingSpan {
        self.start();
        EndingSpan { span: self }
    }

    pub fn record<K: AsKey + ?Sized, V: Value>(&self, key: &K, value: V) {
        if let Some(key) = key.as_key() {
            //let attrs: &dyn AttributeSet = &[(key, &value as &dyn Value)];
            self.record_all(&[(key, &value as &dyn Value)]);
        }
    }

    pub fn record_all<AS: AttributeSet>(&self, attributes: AS) -> &Self {
        let attrs: &dyn AttributeSet = &attributes;
        self.record_data(&attrs);
        self
    }

    pub fn record_data(&self, data: &dyn SpanRecordData) -> &Self {
        self.with_subscriber(|(id, dispatch)| {
            dispatch.record(id, data);
        });
        self
    }

    pub fn follows_from(&self, from: impl Into<Option<Id>>) -> &Self {
        if let Some(from) = from.into() {
            self.with_subscriber(|(id, dispatch)| {
                dispatch.record_follows_from(id, &from);
            });
        }
        self
    }

    pub fn follows_from_data(&self, from: &dyn SpanFollowsFromData) -> &Self {
        self.with_subscriber(|(id, dispatch)| {
            dispatch.record_follows_from(id, from);
        });
        self
    }

    /// Invokes a function with a reference to this span's ID and subscriber.
    ///
    /// if this span is enabled, the provided function is called, and the result is returned.
    /// If the span is disabled, the function is not called, and this method returns `None`
    /// instead.
    pub fn with_subscriber<T>(&self, f: impl FnOnce((&Id, &Dispatch)) -> T) -> Option<T> {
        self.inner
            .as_ref()
            .map(|inner| f((&inner.id, &inner.subscriber)))
    }
}

impl std::hash::Hash for Span {
    fn hash<H: std::hash::Hasher>(&self, hasher: &mut H) {
        self.inner.hash(hasher);
    }
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut span = f.debug_struct("Span");
        if let Some(ref inner) = self.inner {
            span.field("id", &inner.id());
        } else {
            span.field("disabled", &true);
        }

        span.finish()
    }
}

impl<'a> From<&'a Span> for Option<&'a Id> {
    fn from(span: &'a Span) -> Self {
        span.inner.as_ref().map(|inner| &inner.id)
    }
}

impl<'a> From<&'a Span> for Option<Id> {
    fn from(span: &'a Span) -> Self {
        span.inner.as_ref().map(Inner::id)
    }
}

impl From<Span> for Option<Id> {
    fn from(span: Span) -> Self {
        span.inner.as_ref().map(Inner::id)
    }
}

impl Drop for Span {
    #[inline(always)]
    fn drop(&mut self) {
        if let Some(Inner {
            ref id,
            ref subscriber,
        }) = self.inner
        {
            subscriber.try_close(id.clone());
        }
    }
}

/// A guard representing a span which will end when dropped.
#[derive(Debug)]
#[must_use = "an ending guard ends immediatly unless used"]
pub struct EndSpan<'a> {
    span: &'a Span,
}

/// An owned version of [`EndSpan`], a guard representing a span which will
/// be explicitly ended when dropped.
#[derive(Debug)]
#[must_use = "an ending guard ends immediatly unless used"]
pub struct EndingSpan {
    span: Span,
}

impl EndingSpan {
    /// Returns this span's `Id`, if it is enabled.
    pub fn id(&self) -> Option<Id> {
        self.inner.as_ref().map(Inner::id)
    }

    /// Cancels ending this span, returning the underlying [`Span`].
    #[inline]
    pub fn cancel(mut self) -> Span {
        // One does not simply move out of a struct with `Drop`.
        mem::replace(&mut self.span, Span::none())
    }
}

impl Deref for EndingSpan {
    type Target = Span;

    #[inline]
    fn deref(&self) -> &Span {
        &self.span
    }
}

impl Drop for EndSpan<'_> {
    #[inline(always)]
    fn drop(&mut self) {
        self.span.end()
    }
}

impl Drop for EndingSpan {
    #[inline(always)]
    fn drop(&mut self) {
        self.span.end()
    }
}

/// A guard representing a span which has been entered and is currently
/// executing.
///
/// When the guard is dropped, the span will be exited.
///
/// This is returned by the [`Span::enter`] function.
///
/// [`Span::enter`]: super::Span::enter
#[derive(Debug)]
#[must_use = "once a span has been entered, it should be exited"]
pub struct Entered<'a> {
    span: &'a Span,
}

/// An owned version of [`Entered`], a guard representing a span which has been
/// entered and is currently executing.
///
/// When the guard is dropped, the span will be exited.
///
/// This is returned by the [`Span::entered`] function.
///
/// [`Span::entered`]: super::Span::entered()
#[derive(Debug)]
#[must_use = "once a span has been entered, it should be exited"]
pub struct EnteredSpan {
    span: Span,

    /// ```compile_fail
    /// use korio::span::*;
    /// trait AssertSend: Send {}
    ///
    /// impl AssertSend for EnteredSpan {}
    /// ```
    _not_send: PhantomData<dyn NotSend>,
}

impl EnteredSpan {
    /// Returns this span's `Id`, if it is enabled.
    pub fn id(&self) -> Option<Id> {
        self.inner.as_ref().map(Inner::id)
    }

    /// Exits this span, returning the underlying [`Span`].
    #[inline]
    pub fn exit(mut self) -> Span {
        // One does not simply move out of a struct with `Drop`.
        let span = mem::replace(&mut self.span, Span::none());
        span.do_exit();
        span
    }
}

impl Deref for EnteredSpan {
    type Target = Span;

    #[inline]
    fn deref(&self) -> &Span {
        &self.span
    }
}

impl Drop for Entered<'_> {
    #[inline(always)]
    fn drop(&mut self) {
        self.span.do_exit()
    }
}

impl Drop for EnteredSpan {
    #[inline(always)]
    fn drop(&mut self) {
        self.span.do_exit()
    }
}

/// A trait that implements `Sync`, but not `Send`. Used with PhantomData, this
/// can mark a struct as `!Send`.
trait NotSend: Sync {}

pub trait NewSpanDataCreate: NewSpanData + Metadata {
    fn create(&self) -> Span {
        Span::from_data(&self)
    }

    fn create_started(&self) -> Span {
        let ret = self.create();
        ret.start();
        ret
    }
}

impl<T: NewSpanData> NewSpanDataCreate for T {}

pub trait SpanDataCreate: SpanData + Metadata {
    fn create_downstream(&self) -> Span {
        Span::from_downstream(&self)
    }
}

impl<T: SpanData> SpanDataCreate for T {}
