/// Constructs a new span.
///
#[macro_export]
macro_rules! korio_span {
    (target: $target:expr, parent: $parent:expr, $level:expr, $name:expr) => {
        $crate::korio_span!(target: $target, parent: $parent, $level, $name,)
    };
    (target: $target:expr, parent: $parent:expr, $level:expr, $name:expr, $($attributes:tt)*) => {
        {
            let attrs = $crate::attribute_set!($($attributes)*);

            // span with explicit parent
            let builder = $crate::Span::build_child_of(
                $parent,
                $target,
                $name,
                $level,
            ).with_attributes(&attrs);

            $crate::Span::from_data(&builder)
        }
    };

    (target: $target:expr, $level:expr, $name:expr) => {
        $crate::korio_span!(target: $target, $level, $name,)
    };
    (target: $target:expr, $level:expr, $name:expr, $($attributes:tt)*) => {
        {
            let attrs = $crate::attribute_set!($($attributes)*);

            // span with contextual parent
            let builder = $crate::Span::build(
                $target,
                $name,
                $level
            ).with_attributes(&attrs);

            $crate::Span::from_data(&builder)
        }
    };


    (parent: $parent:expr, $level:expr, $name:expr) => {
        $crate::korio_span!(parent: $parent, $level, $name,)
    };
    (parent: $parent:expr, $level:expr, $name:expr, $($attributes:tt)*) => {
        $crate::korio_span!(target: module_path!(), parent: $parent, $level, $name, $($attributes)*)
    };

    ($level:expr, $name:expr) => {
        $crate::korio_span!($level, $name,)
    };
    ($level:expr, $name:expr, $($attributes:tt)*) => {
        $crate::korio_span!(target: module_path!(), $level, $name, $($attributes)*)
    };
}

/// Constructs a new span builder.
///
#[macro_export]
macro_rules! build_span {
    (target: $target:expr, parent: $parent:expr, $level:expr, $name:expr) => {
        $crate::build_span!(target: $target, parent: $parent, $level, $name,)
    };
    (target: $target:expr, parent: $parent:expr, $level:expr, $name:expr, $($attr:expr)?) => {
            // span with explicit parent
            $crate::Span::build_child_of(
                $parent,
                $target,
                $name,
                $level,
            )$(.with_attributes($attr))?
    };

    (target: $target:expr, $level:expr, $name:expr) => {
        $crate::build_span!(target: $target, $level, $name,)
    };
    (target: $target:expr, $level:expr, $name:expr, $($attr:expr)?) => {
        {
            // span with contextual parent
            $crate::Span::build(
                $target,
                $name,
                $level
            )$(.with_attributes($attr))?
        }
    };


    (parent: $parent:expr, $level:expr, $name:expr) => {
        $crate::build_span!(parent: $parent, $level, $name,)
    };
    (parent: $parent:expr, $level:expr, $name:expr, $($attr:expr)?) => {
        $crate::build_span!(target: module_path!(), parent: $parent, $level, $name, $($attr)?)
    };

    ($level:expr, $name:expr) => {
        $crate::build_span!($level, $name,)
    };
    ($level:expr, $name:expr, $($attr:expr)?) => {
        $crate::build_span!(target: module_path!(), $level, $name, $($attr)?)
    };
}

/// Constructs a new event.
#[macro_export]
macro_rules! korio_event {
    // name / target / parent
    (name: $name:expr, target: $target:expr, parent: $parent:expr, $level:expr, { $($attributes:tt)* } )=> ({
        let attrs = $crate::attribute_set!($($attributes)*);

        let builder = $crate::event::RefEventBuilder::new_child_of(
            $parent,
            $target,
            $name,
            $level
        ).with_attributes(&attrs);
        $crate::event::NewEventDataEmit::emit(&builder);
    });
    (name: $name:expr, target: $target:expr, parent: $parent:expr, $level:expr, { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_event!(
            name: $name,
            target: $target,
            parent: $parent,
            $level,
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    (name: $name:expr, target: $target:expr, parent: $parent:expr, $level:expr, $($k:ident).+ = $($attributes:tt)* ) => (
        $crate::korio_event!(name: $name, target: $target, parent: $parent, $level, { $($k).+ = $($attributes)* })
    );
    (name: $name:expr, target: $target:expr, parent: $parent:expr, $level:expr, $($arg:tt)+) => (
        $crate::korio_event!(name: $name, target: $target, parent: $parent, $level, { $($arg)+ })
    );

    // name / target
    (name: $name:expr, target: $target:expr, $level:expr, { $($attributes:tt)* } )=> ({
        let attrs = $crate::attribute_set!($($attributes)*);

        let builder = $crate::event::RefEventBuilder::new(
            $target,
            $name,
            $level
        ).with_attributes(&attrs);
        $crate::event::NewEventDataEmit::emit(&builder);
    });
    (name: $name:expr, target: $target:expr, $level:expr, { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_event!(
            name: $name,
            target: $target,
            $level,
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    (name: $name:expr, target: $target:expr, $level:expr, $($k:ident).+ = $($attributes:tt)* ) => (
        $crate::korio_event!(name: $name, target: $target, $level, { $($k).+ = $($attributes)* })
    );
    (name: $name:expr, target: $target:expr, $level:expr, $($arg:tt)+) => (
        $crate::korio_event!(name: $name, target: $target, $level, { $($arg)+ })
    );

    // target / parent
    (target: $target:expr, parent: $parent:expr, $level:expr, { $($attributes:tt)* } )=> ({
        let attrs = $crate::attribute_set!($($attributes)*);

        let builder = $crate::event::RefEventBuilder::new_child_of(
            $parent,
            $target,
            $crate::__macro_support::concat!(
                "event ",
                $crate::__macro_support::file!(),
                ":",
                $crate::__macro_support::line!()
            ),
            $level
        ).with_attributes(&attrs);
        $crate::event::NewEventDataEmit::emit(&builder);
    });
    (target: $target:expr, parent: $parent:expr, $level:expr, { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_event!(
            target: $target,
            parent: $parent,
            $level,
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    (target: $target:expr, parent: $parent:expr, $level:expr, $($k:ident).+ = $($attributes:tt)* ) => (
        $crate::korio_event!(target: $target, parent: $parent, $level, { $($k).+ = $($attributes)* })
    );
    (target: $target:expr, parent: $parent:expr, $level:expr, $($arg:tt)+) => (
        $crate::korio_event!(target: $target, parent: $parent, $level, { $($arg)+ })
    );

    // name / parent
    (name: $name:expr, parent: $parent:expr, $level:expr, { $($attributes:tt)* } )=> ({
        let attrs = $crate::attribute_set!($($attributes)*);

        let builder = $crate::event::RefEventBuilder::new_child_of(
            $parent,
            module_path!(),
            $name,
            $level
        ).with_attributes(&attrs);
        $crate::event::NewEventDataEmit::emit(&builder);
    });
    (name: $name:expr, parent: $parent:expr, $level:expr, { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_event!(
            name: $name,
            parent: $parent,
            $level,
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    (name: $name:expr, parent: $parent:expr, $level:expr, $($k:ident).+ = $($attributes:tt)* ) => (
        $crate::korio_event!(name: $name, parent: $parent, $level, { $($k).+ = $($attributes)* })
    );
    (name: $name:expr, parent: $parent:expr, $level:expr, $($arg:tt)+) => (
        $crate::korio_event!(name: $name, parent: $parent, $level, { $($arg)+ })
    );

    // name
    (name: $name:expr, $level:expr, { $($attributes:tt)* } )=> ({
        let attrs = $crate::attribute_set!($($attributes)*);

        let builder = $crate::event::RefEventBuilder::new(
            module_path!(),
            $name,
            $level
        ).with_attributes(&attrs);
        $crate::event::NewEventDataEmit::emit(&builder);
    });
    (name: $name:expr, $level:expr, { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_event!(
            name: $name,
            $level,
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    (name: $name:expr, $level:expr, $($k:ident).+ = $($attributes:tt)* ) => (
        $crate::korio_event!(name: $name, $level, { $($k).+ = $($attributes)* })
    );
    (name: $name:expr, $level:expr, $($arg:tt)+ ) => (
        $crate::korio_event!(name: $name, $level, { $($arg)+ })
    );

    // target
    (target: $target:expr, $level:expr, { $($attributes:tt)* } )=> ({
        let attrs = $crate::attribute_set!($($attributes)*);

        let builder = $crate::event::RefEventBuilder::new(
            $target,
            $crate::__macro_support::concat!(
                "event ",
                $crate::__macro_support::file!(),
                ":",
                $crate::__macro_support::line!()
            ),
            $level
        ).with_attributes(&attrs);
        $crate::event::NewEventDataEmit::emit(&builder);
    });
    (target: $target:expr, $level:expr, { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_event!(
            target: $target,
            $level,
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    (target: $target:expr, $level:expr, $($k:ident).+ = $($attributes:tt)* ) => (
        $crate::korio_event!(target: $target, $level, { $($k).+ = $($attributes)* })
    );
    (target: $target:expr, $level:expr, $($arg:tt)+ ) => (
        $crate::korio_event!(target: $target, $level, { $($arg)+ })
    );

    // parent
    (parent: $parent:expr, $level:expr, { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_event!(
            target: module_path!(),
            parent: $parent,
            $level,
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    (parent: $parent:expr, $level:expr, $($k:ident).+ = $($attribute:tt)*) => (
        $crate::korio_event!(
            target: module_path!(),
            parent: $parent,
            $level,
            { $($k).+ = $($attribute)*}
        )
    );
    (parent: $parent:expr, $level:expr, ?$($k:ident).+ = $($attribute:tt)*) => (
        $crate::korio_event!(
            target: module_path!(),
            parent: $parent,
            $level,
            { ?$($k).+ = $($attribute)*}
        )
    );
    (parent: $parent:expr, $level:expr, %$($k:ident).+ = $($attribute:tt)*) => (
        $crate::korio_event!(
            target: module_path!(),
            parent: $parent,
            $level,
            { %$($k).+ = $($attribute)*}
        )
    );
    (parent: $parent:expr, $level:expr, $($k:ident).+, $($attribute:tt)*) => (
        $crate::korio_event!(
            target: module_path!(),
            parent: $parent,
            $level,
            { $($k).+, $($attribute)*}
        )
    );
    (parent: $parent:expr, $level:expr, %$($k:ident).+, $($attribute:tt)*) => (
        $crate::korio_event!(
            target: module_path!(),
            parent: $parent,
            $level,
            { %$($k).+, $($attribute)*}
        )
    );
    (parent: $parent:expr, $level:expr, ?$($k:ident).+, $($attribute:tt)*) => (
        $crate::korio_event!(
            target: module_path!(),
            parent: $parent,
            $level,
            { ?$($k).+, $($attribute)*}
        )
    );
    (parent: $parent:expr, $level:expr, $($arg:tt)+ ) => (
        $crate::korio_event!(target: module_path!(), parent: $parent, $level, { $($arg)+ })
    );

    // ...
    ( $level:expr, { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_event!(
            target: module_path!(),
            $level,
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    ( $level:expr, { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_event!(
            target: module_path!(),
            $level,
            { message = format_args!($($arg)+), $($attributes)* }
        )
    );
    ($level:expr, $($k:ident).+ = $($attribute:tt)*) => (
        $crate::korio_event!(
            target: module_path!(),
            $level,
            { $($k).+ = $($attribute)*}
        )
    );
    ($level:expr, $($k:ident).+, $($attribute:tt)*) => (
        $crate::korio_event!(
            target: module_path!(),
            $level,
            { $($k).+, $($attribute)*}
        )
    );
    ($level:expr, ?$($k:ident).+, $($attribute:tt)*) => (
        $crate::korio_event!(
            target: module_path!(),
            $level,
            { ?$($k).+, $($attribute)*}
        )
    );
    ($level:expr, %$($k:ident).+, $($attribute:tt)*) => (
        $crate::korio_event!(
            target: module_path!(),
            $level,
            { %$($k).+, $($attribute)*}
        )
    );
    ($level:expr, ?$($k:ident).+) => (
        $crate::korio_event!($level, ?$($k).+,)
    );
    ($level:expr, %$($k:ident).+) => (
        $crate::korio_event!($level, %$($k).+,)
    );
    ($level:expr, $($k:ident).+) => (
        $crate::korio_event!($level, $($k).+,)
    );
    ( $level:expr, $($arg:tt)+ ) => (
        $crate::korio_event!(target: module_path!(), $level, { $($arg)+ })
    );
}

/// Constructs a new event builder.
///
#[macro_export]
macro_rules! build_event {
    // name / target / parent
    (name: $name:expr, target: $target:expr, parent: $parent:expr, $level:expr, { $($attributes:tt)* } ) => {
        $crate::event::RefEventBuilder::new_child_of(
            $parent,
            $target,
            $name,
            $level
        );
        $crate::attributes!(builder, $($attributes)*)

        $crate::build_event!(target: $target, parent: $parent, $level, $name,)
    };
    (target: $target:expr, parent: $parent:expr, $level:expr, $name:expr) => {
        $crate::build_event!(target: $target, parent: $parent, $level, $name,)
    };
    (target: $target:expr, parent: $parent:expr, $level:expr, $name:expr, $($attributes:tt)*) => {{
        let builder = $crate::event::RefEventBuilder::new_child_of(
            $parent,
            $target,
            $name,
            $level
        );
        $crate::attributes!(builder, $($attributes)*)
    }};

    (target: $target:expr, $level:expr, $name:expr) => {
        $crate::build_event!(target: $target, $level, $name,)
    };
    (target: $target:expr, $level:expr, $name:expr, $($attributes:tt)*) => {{
        let builder = $crate::event::RefEventBuilder::new(
            $target,
            $name,
            $level
        );
        $crate::attributes!(builder, $($attributes)*)
    }};


    (parent: $parent:expr, $level:expr, $name:expr) => {
        $crate::build_event!(parent: $parent, $level, $name,)
    };
    (parent: $parent:expr, $level:expr, $name:expr, $($attributes:tt)*) => {
        $crate::build_event!(target: module_path!(), parent: $parent, $level, $name, $($attributes)*)
    };

    ($level:expr, $name:expr) => {
        $crate::build_event!($level, $name,)
    };
    ($level:expr, $name:expr, $($attributes:tt)*) => {
        $crate::build_event
        !(target: module_path!(), $level, $name, $($attributes)*)
    };
}

#[macro_export]
macro_rules! korio_record_all {
    ($span:expr, $($attributes:tt)*) => {
        if let Some(id) = $span.id() {
            let attrs = $crate::attribute_set!($($attributes)*);
            $span.record_all(attrs);
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! attribute_set2 {
    // === base case ===
    (@ { $(,)* $(($k:expr, $val:expr)),* $(,)* } $(,)* ) => ([
        $((
            &*$k,
            $val,
        ),)*
    ]);

    // === recursive case (more tts) ===

    (@ { $(,)* $(($attrk:expr, $attrv:expr)),* }, $($k:ident).+ = ?$val:expr, $($rest:tt)* ) => {
        $crate::attribute_set2!(@ { $(($attrk, $attrv)),*, ($crate::__korio_stringify!($($k).+), &$crate::metadata::debug(&$val) as &dyn $crate::metadata::Value ) }, $($rest)* )
    };
    (@ { $(,)* $(($attrk:expr, $attrv:expr)),* }, $($k:ident).+ = %$val:expr, $($rest:tt)* ) => {
        $crate::attribute_set2!(@ { $(($attrk, $attrv)),*, ($crate::__korio_stringify!($($k).+), &$crate::metadata::display(&$val) as &dyn $crate::metadata::Value ) }, $($rest)* )
    };
    (@ { $(,)* $(($attrk:expr, $attrv:expr)),* }, $($k:ident).+ = $val:expr, $($rest:tt)* ) => {
        $crate::attribute_set2!(@ { $(($attrk, $attrv)),*, ($crate::__korio_stringify!($($k).+), &$val as &dyn $crate::metadata::Value ) }, $($rest)* )
    };
    (@ { $(,)* $(($attrk:expr, $attrv:expr)),* }, ?$($k:ident).+, $($rest:tt)* ) => {
        $crate::attribute_set2!(@ { $(($attrk, $attrv)),*, ($crate::__korio_stringify!($($k).+), &$crate::metadata::debug(&$($k).+) as &dyn $crate::metadata::Value ) }, $($rest)* )
    };
    (@ { $(,)* $(($attrk:expr, $attrv:expr)),* }, %$($k:ident).+, $($rest:tt)* ) => {
        $crate::attribute_set2!(@ { $(($attrk, $attrv)),*, ($crate::__korio_stringify!($($k).+), &$crate::metadata::display(&$($k).+) as &dyn $crate::metadata::Value ) }, $($rest)* )
    };
    (@ { $(,)* $(($attrk:expr, $attrv:expr)),* }, $($k:ident).+, $($rest:tt)* ) => {
        $crate::attribute_set2!(@ { $(($attrk, $attrv)),*, ($crate::__korio_stringify!($($k).+), &$($k).+ as &dyn $crate::metadata::Value ) }, $($rest)* )
    };

    (@ { $(,)* $(($attrk:expr, $attrv:expr)),* }, $($k:ident).+ = ?$val:expr ) => {
        $crate::attribute_set2!(@ { $(($attrk, $attrv)),*, ($crate::__korio_stringify!($($k).+), &$crate::metadata::debug(&$val) as &dyn $crate::metadata::Value ) }, )
    };
    (@ { $(,)* $(($attrk:expr, $attrv:expr)),* }, $($k:ident).+ = %$val:expr ) => {
        $crate::attribute_set2!(@ { $(($attrk, $attrv)),*, ($crate::__korio_stringify!($($k).+), &$crate::metadata::display(&$val) as &dyn $crate::metadata::Value ) }, )
    };
    (@ { $(,)* $(($attrk:expr, $attrv:expr)),* }, $($k:ident).+ = $val:expr ) => {
        $crate::attribute_set2!(@ { $(($attrk, $attrv)),*, ($crate::__korio_stringify!($($k).+), &$val as &dyn $crate::metadata::Value ) }, )
    };
    (@ { $(,)* $(($attrk:expr, $attrv:expr)),* }, ?$($k:ident).+ ) => {
        $crate::attribute_set2!(@ { $(($attrk, $attrv)),*, ($crate::__korio_stringify!($($k).+), &$crate::metadata::debug(&$($k).+) as &dyn $crate::metadata::Value ) }, )
    };
    (@ { $(,)* $(($attrk:expr, $attrv:expr)),* }, %$($k:ident).+ ) => {
        $crate::attribute_set2!(@ { $(($attrk, $attrv)),*, ($crate::__korio_stringify!($($k).+), &$crate::metadata::display(&$($k).+) as &dyn $crate::metadata::Value ) }, )
    };
    (@ { $(,)* $(($attrk:expr, $attrv:expr)),* }, $($k:ident).+ ) => {
        $crate::attribute_set2!(@ { $(($attrk, $attrv)),*, ($crate::__korio_stringify!($($k).+), &$($k).+ as &dyn $crate::metadata::Value ) }, )
    };

    // Handle literal names
    (@ { $(,)* $(($attrk:expr, $attrv:expr)),* }, $k:literal = ?$val:expr, $($rest:tt)* ) => {
        $crate::attribute_set2!(@ { $(($attrk, $attrv)),*, ($crate::metadata::KeyBuf::from_static($k), &$crate::metadata::debug(&$val) as &dyn $crate::metadata::Value ) }, $($rest)* )
    };
    (@ { $(,)* $(($attrk:expr, $attrv:expr)),* }, $k:literal = %$val:expr, $($rest:tt)* ) => {
        $crate::attribute_set2!(@ { $(($attrk, $attrv)),*, ($crate::metadata::KeyBuf::from_static($k), &$crate::metadata::display(&$val) as &dyn $crate::metadata::Value ) }, $($rest)* )
    };
    (@ { $(,)* $(($attrk:expr, $attrv:expr)),* }, $k:literal = $val:expr, $($rest:tt)* ) => {
        $crate::attribute_set2!(@ { $(($attrk, $attrv)),*, ($crate::metadata::KeyBuf::from_static($k), &$val as &dyn $crate::metadata::Value ) }, $($rest)* )
    };

    (@ { $(,)* $(($attrk:expr, $attrv:expr)),* }, $k:literal = ?$val:expr ) => {
        $crate::attribute_set2!(@ { $(($attrk, $attrv)),*, ($crate::metadata::KeyBuf::from_static($k), &$crate::metadata::debug(&$val) as &dyn $crate::metadata::Value ) }, )
    };
    (@ { $(,)* $(($attrk:expr, $attrv:expr)),* }, $k:literal = %$val:expr ) => {
        $crate::attribute_set2!(@ { $(($attrk, $attrv)),*, ($crate::metadata::KeyBuf::from_static($k), &$crate::metadata::display(&$val) as &dyn $crate::metadata::Value ) }, )
    };
    (@ { $(,)* $(($attrk:expr, $attrv:expr)),* }, $k:literal = $val:expr ) => {
        $crate::attribute_set2!(@ { $(($attrk, $attrv)),*, ($crate::metadata::KeyBuf::from_static($k), &$val as &dyn $crate::metadata::Value ) }, )
    };

    // Handle constant names
    (@ { $(,)* $(($attrk:expr, $attrv:expr)),* }, { $k:expr } = ?$val:expr, $($rest:tt)* ) => {
        $crate::attribute_set2!(@ { $(($attrk, $attrv)),*, ($k, &$crate::metadata::debug(&$val) as &dyn $crate::metadata::Value ) }, $($rest)* )
    };
    (@ { $(,)* $(($attrk:expr, $attrv:expr)),* }, { $k:expr } = %$val:expr, $($rest:tt)* ) => {
        $crate::attribute_set2!(@ { $(($attrk, $attrv)),*, ($k, &$crate::metadata::display(&$val) as &dyn $crate::metadata::Value ) }, $($rest)* )
    };
    (@ { $(,)* $(($attrk:expr, $attrv:expr)),* }, { $k:expr } = $val:expr, $($rest:tt)* ) => {
        $crate::attribute_set2!(@ { $(($attrk, $attrv)),*, ($k, &$val as &dyn $crate::metadata::Value ) }, $($rest)* )
    };

    (@ { $(,)* $(($attrk:expr, $attrv:expr)),* }, { $k:expr } = ?$val:expr ) => {
        $crate::attribute_set2!(@ { $(($attrk, $attrv)),*, ($k, &$crate::metadata::debug(&$val) as &dyn $crate::metadata::Value ) }, )
    };
    (@ { $(,)* $(($attrk:expr, $attrv:expr)),* }, { $k:expr } = %$val:expr ) => {
        $crate::attribute_set2!(@ { $(($attrk, $attrv)),*, ($k, &$crate::metadata::display(&$val) as &dyn $crate::metadata::Value ) }, )
    };
    (@ { $(,)* $(($attrk:expr, $attrv:expr)),* }, { $k:expr } = $val:expr ) => {
        $crate::attribute_set2!(@ { $(($attrk, $attrv)),*, ($k, &$val as &dyn $crate::metadata::Value ) }, )
    };

    // Remainder is unparsable, but exists --- must be format args!
    (@ { $(,)* $(($attrk:expr, $attrv:expr)),* }, $($rest:tt)+) => {
        $crate::attribute_set2!(@ { ($crate::metadata::KeyBuf::from_static("message"), &$crate::__macro_support::format_args!($($rest)+) as &dyn $crate::metadata::Value), $(($attrk, $attrv)),* },)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! attribute_set {
    // === entry ===
    ($($kvs:tt)+) => {
        {
            $crate::attribute_set2!(
                @ { },
                $($kvs)+
            )
        }
    };
    () => {[]};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __korio_stringify {
    ($($k:ident).+) => {
        $crate::metadata::KeyBuf::from_static(stringify!($($k).+))
    }
}

#[macro_export]
macro_rules! korio_trace_span {
    (target: $target:expr, parent: $parent:expr, $name:expr) => {
        $crate::korio_trace_span!(target: $target, parent: $parent, $name,)
    };
    (target: $target:expr, parent: $parent:expr, $name:expr, $($attributes:tt)*) => {
        $crate::korio_span!(target: $target, parent: $parent, $crate::metadata::Severity::Trace, $name, $($attributes)*)
    };

    (target: $target:expr, $name:expr) => {
        $crate::korio_trace_span!(target: $target, $name,)
    };
    (target: $target:expr, $name:expr, $($attributes:tt)*) => {
        $crate::korio_span!(target: $target, $crate::metadata::Severity::Trace, $name, $($attributes)*)
    };


    (parent: $parent:expr, $name:expr) => {
        $crate::korio_trace_span!(parent: $parent, $name,)
    };
    (parent: $parent:expr, $name:expr, $($attributes:tt)*) => {
        $crate::korio_trace_span!(target: module_path!(), parent: $parent, $name, $($attributes)*)
    };

    ($name:expr) => {
        $crate::korio_trace_span!($name,)
    };
    ($name:expr, $($attributes:tt)*) => {
        $crate::korio_trace_span!(target: module_path!(), $name, $($attributes)*)
    };
}

#[macro_export]
macro_rules! korio_debug_span {
    (target: $target:expr, parent: $parent:expr, $name:expr) => {
        $crate::korio_debug_span!(target: $target, parent: $parent, $name,)
    };
    (target: $target:expr, parent: $parent:expr, $name:expr, $($attributes:tt)*) => {
        $crate::korio_span!(target: $target, parent: $parent, $crate::metadata::Severity::Debug, $name, $($attributes)*)
    };

    (target: $target:expr, $name:expr) => {
        $crate::korio_debug_span!(target: $target, $name,)
    };
    (target: $target:expr, $name:expr, $($attributes:tt)*) => {
        $crate::korio_span!(target: $target, $crate::metadata::Severity::Debug, $name, $($attributes)*)
    };


    (parent: $parent:expr, $name:expr) => {
        $crate::korio_debug_span!(parent: $parent, $name,)
    };
    (parent: $parent:expr, $name:expr, $($attributes:tt)*) => {
        $crate::korio_debug_span!(target: module_path!(), parent: $parent, $name, $($attributes)*)
    };

    ($name:expr) => {
        $crate::korio_debug_span!($name,)
    };
    ($name:expr, $($attributes:tt)*) => {
        $crate::korio_debug_span!(target: module_path!(), $name, $($attributes)*)
    };
}

#[macro_export]
macro_rules! korio_info_span {
    (target: $target:expr, parent: $parent:expr, $name:expr) => {
        $crate::korio_info_span!(target: $target, parent: $parent, $name,)
    };
    (target: $target:expr, parent: $parent:expr, $name:expr, $($attributes:tt)*) => {
        $crate::korio_span!(target: $target, parent: $parent, $crate::metadata::Severity::Info, $name, $($attributes)*)
    };

    (target: $target:expr, $name:expr) => {
        $crate::korio_info_span!(target: $target, $name,)
    };
    (target: $target:expr, $name:expr, $($attributes:tt)*) => {
        $crate::korio_span!(target: $target, $crate::metadata::Severity::Info, $name, $($attributes)*)
    };

    (parent: $parent:expr, $name:expr) => {
        $crate::korio_info_span!(parent: $parent, $name,)
    };
    (parent: $parent:expr, $name:expr, $($attributes:tt)*) => {
        $crate::korio_info_span!(target: module_path!(), parent: $parent, $name, $($attributes)*)
    };

    ($name:expr) => {
        $crate::korio_info_span!($name,)
    };
    ($name:expr, $($attributes:tt)*) => {
        $crate::korio_info_span!(target: module_path!(), $name, $($attributes)*)
    };
}

#[macro_export]
macro_rules! korio_warn_span {
    (target: $target:expr, parent: $parent:expr, $name:expr) => {
        $crate::korio_warn_span!(target: $target, parent: $parent, $name,)
    };
    (target: $target:expr, parent: $parent:expr, $name:expr, $($attributes:tt)*) => {
        $crate::korio_span!(target: $target, parent: $parent, $crate::metadata::Severity::Warn, $name, $($attributes)*)
    };

    (target: $target:expr, $name:expr) => {
        $crate::korio_warn_span!(target: $target, $name,)
    };
    (target: $target:expr, $name:expr, $($attributes:tt)*) => {
        $crate::korio_span!(target: $target, $crate::metadata::Severity::Warn, $name, $($attributes)*)
    };

    (parent: $parent:expr, $name:expr) => {
        $crate::korio_warn_span!(parent: $parent, $name,)
    };
    (parent: $parent:expr, $name:expr, $($attributes:tt)*) => {
        $crate::korio_warn_span!(target: module_path!(), parent: $parent, $name, $($attributes)*)
    };

    ($name:expr) => {
        $crate::korio_warn_span!($name,)
    };
    ($name:expr, $($attributes:tt)*) => {
        $crate::korio_warn_span!(target: module_path!(), $name, $($attributes)*)
    };
}

#[macro_export]
macro_rules! korio_error_span {
    (target: $target:expr, parent: $parent:expr, $name:expr) => {
        $crate::korio_error_span!(target: $target, parent: $parent, $name,)
    };
    (target: $target:expr, parent: $parent:expr, $name:expr, $($attributes:tt)*) => {
        $crate::korio_span!(target: $target, parent: $parent, $crate::metadata::Severity::Error, $name, $($attributes)*)
    };

    (target: $target:expr, $name:expr) => {
        $crate::korio_error_span!(target: $target, $name,)
    };
    (target: $target:expr, $name:expr, $($attributes:tt)*) => {
        $crate::korio_span!(target: $target, $crate::metadata::Severity::Error, $name, $($attributes)*)
    };

    (parent: $parent:expr, $name:expr) => {
        $crate::korio_error_span!(parent: $parent, $name,)
    };
    (parent: $parent:expr, $name:expr, $($attributes:tt)*) => {
        $crate::korio_error_span!(target: module_path!(), parent: $parent, $name, $($attributes)*)
    };

    ($name:expr) => {
        $crate::korio_error_span!($name,)
    };
    ($name:expr, $($attributes:tt)*) => {
        $crate::korio_error_span!(target: module_path!(), $name, $($attributes)*)
    };
}

/// Construct a new event at the trace severity.
///
#[macro_export]
macro_rules! korio_trace {
    // name / target / parent
    (name: $name:expr, target: $target:expr, parent: $parent:expr, { $($attributes:tt)* } )=> (
        $crate::korio_event!(name: $name, target: $target, parent: $parent, $crate::Severity::Trace, { $($attributes)* })
    );
    (name: $name:expr, target: $target:expr, parent: $parent:expr, { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_trace!(
            name: $name,
            target: $target,
            parent: $parent,
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    (name: $name:expr, target: $target:expr, parent: $parent:expr, $($k:ident).+ = $($attributes:tt)* ) => (
        $crate::korio_trace!(name: $name, target: $target, parent: $parent, { $($k).+ = $($attributes)* })
    );
    (name: $name:expr, target: $target:expr, parent: $parent:expr, $($arg:tt)+) => (
        $crate::korio_trace!(name: $name, target: $target, parent: $parent, { $($arg)+ })
    );

    // name / target
    (name: $name:expr, target: $target:expr, { $($attributes:tt)* } )=> (
        $crate::korio_event!(name: $name, target: $target, $crate::Severity::Trace, { $($attributes)* })
    );
    (name: $name:expr, target: $target:expr, { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_trace!(
            name: $name,
            target: $target,
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    (name: $name:expr, target: $target:expr, $($k:ident).+ = $($attributes:tt)* ) => (
        $crate::korio_trace!(name: $name, target: $target, { $($k).+ = $($attributes)* })
    );
    (name: $name:expr, target: $target:expr, $($arg:tt)+) => (
        $crate::korio_trace!(name: $name, target: $target, { $($arg)+ })
    );

    // target / parent
    (target: $target:expr, parent: $parent:expr, { $($attributes:tt)* } )=> (
        $crate::korio_event!(target: $target, parent: $parent, $crate::Severity::Trace, { $($attributes)* })
    );
    (target: $target:expr, parent: $parent:expr, { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_trace!(
            target: $target,
            parent: $parent,
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    (target: $target:expr, parent: $parent:expr, $($k:ident).+ = $($attributes:tt)* ) => (
        $crate::korio_trace!(target: $target, parent: $parent, { $($k).+ = $($attributes)* })
    );
    (target: $target:expr, parent: $parent:expr, $($arg:tt)+) => (
        $crate::korio_trace!(target: $target, parent: $parent, { $($arg)+ })
    );

    // name / parent
    (name: $name:expr, parent: $parent:expr, { $($attributes:tt)* } )=> (
        $crate::korio_event!(name: $name, parent: $parent, $crate::Severity::Trace, { $($attributes)* })
    );
    (name: $name:expr, parent: $parent:expr, { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_trace!(
            name: $name,
            parent: $parent,
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    (name: $name:expr, parent: $parent:expr, $($k:ident).+ = $($attributes:tt)* ) => (
        $crate::korio_trace!(name: $name, parent: $parent, { $($k).+ = $($attributes)* })
    );
    (name: $name:expr, parent: $parent:expr, $($arg:tt)+) => (
        $crate::korio_trace!(name: $name, parent: $parent, { $($arg)+ })
    );

    // name
    (name: $name:expr, { $($attributes:tt)* } )=> (
        $crate::korio_event!(name: $name, $crate::Severity::Trace, { $($attributes)* })
    );
    (name: $name:expr, { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_trace!(
            name: $name,
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    (name: $name:expr, $($k:ident).+ = $($attributes:tt)* ) => (
        $crate::korio_trace!(name: $name, { $($k).+ = $($attributes)* })
    );
    (name: $name:expr, $($arg:tt)+ ) => (
        $crate::korio_trace!(name: $name, { $($arg)+ })
    );

    // target
    (target: $target:expr, { $($attributes:tt)* } )=> (
        $crate::korio_event!(target: $target, $crate::Severity::Trace, { $($attributes)* })
    );
    (target: $target:expr, { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_trace!(
            target: $target,
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    (target: $target:expr, $($k:ident).+ = $($attributes:tt)* ) => (
        $crate::korio_trace!(target: $target, { $($k).+ = $($attributes)* })
    );
    (target: $target:expr, $($arg:tt)+ ) => (
        $crate::korio_trace!(target: $target, { $($arg)+ })
    );

    // parent
    (parent: $parent:expr, { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_trace!(
            target: module_path!(),
            parent: $parent,
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    (parent: $parent:expr, $($k:ident).+ = $($attribute:tt)*) => (
        $crate::korio_trace!(
            target: module_path!(),
            parent: $parent,
            { $($k).+ = $($attribute)*}
        )
    );
    (parent: $parent:expr, ?$($k:ident).+ = $($attribute:tt)*) => (
        $crate::korio_trace!(
            target: module_path!(),
            parent: $parent,
            { ?$($k).+ = $($attribute)*}
        )
    );
    (parent: $parent:expr, %$($k:ident).+ = $($attribute:tt)*) => (
        $crate::korio_trace!(
            target: module_path!(),
            parent: $parent,
            { %$($k).+ = $($attribute)*}
        )
    );
    (parent: $parent:expr, $($k:ident).+, $($attribute:tt)*) => (
        $crate::korio_trace!(
            target: module_path!(),
            parent: $parent,
            { $($k).+, $($attribute)*}
        )
    );
    (parent: $parent:expr, %$($k:ident).+, $($attribute:tt)*) => (
        $crate::korio_trace!(
            target: module_path!(),
            parent: $parent,
            { %$($k).+, $($attribute)*}
        )
    );
    (parent: $parent:expr, ?$($k:ident).+, $($attribute:tt)*) => (
        $crate::korio_trace!(
            target: module_path!(),
            parent: $parent,
            { ?$($k).+, $($attribute)*}
        )
    );
    (parent: $parent:expr, $($arg:tt)+ ) => (
        $crate::korio_trace!(target: module_path!(), parent: $parent, { $($arg)+ })
    );

    // ...
    ( { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_trace!(
            target: module_path!(),
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    ( { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_trace!(
            target: module_path!(),
            { message = format_args!($($arg)+), $($attributes)* }
        )
    );
    ($($k:ident).+ = $($attribute:tt)*) => (
        $crate::korio_trace!(
            target: module_path!(),
            { $($k).+ = $($attribute)*}
        )
    );
    ($($k:ident).+, $($attribute:tt)*) => (
        $crate::korio_trace!(
            target: module_path!(),
            { $($k).+, $($attribute)*}
        )
    );
    (?$($k:ident).+, $($attribute:tt)*) => (
        $crate::korio_trace!(
            target: module_path!(),
            { ?$($k).+, $($attribute)*}
        )
    );
    (%$($k:ident).+, $($attribute:tt)*) => (
        $crate::korio_trace!(
            target: module_path!(),
            { %$($k).+, $($attribute)*}
        )
    );
    (?$($k:ident).+) => (
        $crate::korio_trace!(?$($k).+,)
    );
    (%$($k:ident).+) => (
        $crate::korio_trace!(%$($k).+,)
    );
    ($($k:ident).+) => (
        $crate::korio_trace!($($k).+,)
    );
    ( $($arg:tt)+ ) => (
        $crate::korio_trace!(target: module_path!(), { $($arg)+ })
    );
}

/// Construct a new event at the debug severity.
///
#[macro_export]
macro_rules! korio_debug {
    // name / target / parent
    (name: $name:expr, target: $target:expr, parent: $parent:expr, { $($attributes:tt)* } )=> (
        $crate::korio_event!(name: $name, target: $target, parent: $parent, $crate::Severity::Debug, { $($attributes)* })
    );
    (name: $name:expr, target: $target:expr, parent: $parent:expr, { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_debug!(
            name: $name,
            target: $target,
            parent: $parent,
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    (name: $name:expr, target: $target:expr, parent: $parent:expr, $($k:ident).+ = $($attributes:tt)* ) => (
        $crate::korio_debug!(name: $name, target: $target, parent: $parent, { $($k).+ = $($attributes)* })
    );
    (name: $name:expr, target: $target:expr, parent: $parent:expr, $($arg:tt)+) => (
        $crate::korio_debug!(name: $name, target: $target, parent: $parent, { $($arg)+ })
    );

    // name / target
    (name: $name:expr, target: $target:expr, { $($attributes:tt)* } )=> (
        $crate::korio_event!(name: $name, target: $target, $crate::Severity::Debug, { $($attributes)* })
    );
    (name: $name:expr, target: $target:expr, { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_debug!(
            name: $name,
            target: $target,
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    (name: $name:expr, target: $target:expr, $($k:ident).+ = $($attributes:tt)* ) => (
        $crate::korio_debug!(name: $name, target: $target, { $($k).+ = $($attributes)* })
    );
    (name: $name:expr, target: $target:expr, $($arg:tt)+) => (
        $crate::korio_debug!(name: $name, target: $target, { $($arg)+ })
    );

    // target / parent
    (target: $target:expr, parent: $parent:expr, { $($attributes:tt)* } )=> (
        $crate::korio_event!(target: $target, parent: $parent, $crate::Severity::Debug, { $($attributes)* })
    );
    (target: $target:expr, parent: $parent:expr, { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_debug!(
            target: $target,
            parent: $parent,
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    (target: $target:expr, parent: $parent:expr, $($k:ident).+ = $($attributes:tt)* ) => (
        $crate::korio_debug!(target: $target, parent: $parent, { $($k).+ = $($attributes)* })
    );
    (target: $target:expr, parent: $parent:expr, $($arg:tt)+) => (
        $crate::korio_debug!(target: $target, parent: $parent, { $($arg)+ })
    );

    // name / parent
    (name: $name:expr, parent: $parent:expr, { $($attributes:tt)* } )=> (
        $crate::korio_event!(name: $name, parent: $parent, $crate::Severity::Debug, { $($attributes)* })
    );
    (name: $name:expr, parent: $parent:expr, { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_debug!(
            name: $name,
            parent: $parent,
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    (name: $name:expr, parent: $parent:expr, $($k:ident).+ = $($attributes:tt)* ) => (
        $crate::korio_debug!(name: $name, parent: $parent, { $($k).+ = $($attributes)* })
    );
    (name: $name:expr, parent: $parent:expr, $($arg:tt)+) => (
        $crate::korio_debug!(name: $name, parent: $parent, { $($arg)+ })
    );

    // name
    (name: $name:expr, { $($attributes:tt)* } )=> (
        $crate::korio_event!(name: $name, $crate::Severity::Debug, { $($attributes)* })
    );
    (name: $name:expr, { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_debug!(
            name: $name,
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    (name: $name:expr, $($k:ident).+ = $($attributes:tt)* ) => (
        $crate::korio_debug!(name: $name, { $($k).+ = $($attributes)* })
    );
    (name: $name:expr, $($arg:tt)+ ) => (
        $crate::korio_debug!(name: $name, { $($arg)+ })
    );

    // target
    (target: $target:expr, { $($attributes:tt)* } )=> (
        $crate::korio_event!(target: $target, $crate::Severity::Debug, { $($attributes)* })
    );
    (target: $target:expr, { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_debug!(
            target: $target,
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    (target: $target:expr, $($k:ident).+ = $($attributes:tt)* ) => (
        $crate::korio_debug!(target: $target, { $($k).+ = $($attributes)* })
    );
    (target: $target:expr, $($arg:tt)+ ) => (
        $crate::korio_debug!(target: $target, { $($arg)+ })
    );

    // parent
    (parent: $parent:expr, { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_debug!(
            target: module_path!(),
            parent: $parent,
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    (parent: $parent:expr, $($k:ident).+ = $($attribute:tt)*) => (
        $crate::korio_debug!(
            target: module_path!(),
            parent: $parent,
            { $($k).+ = $($attribute)*}
        )
    );
    (parent: $parent:expr, ?$($k:ident).+ = $($attribute:tt)*) => (
        $crate::korio_debug!(
            target: module_path!(),
            parent: $parent,
            { ?$($k).+ = $($attribute)*}
        )
    );
    (parent: $parent:expr, %$($k:ident).+ = $($attribute:tt)*) => (
        $crate::korio_debug!(
            target: module_path!(),
            parent: $parent,
            { %$($k).+ = $($attribute)*}
        )
    );
    (parent: $parent:expr, $($k:ident).+, $($attribute:tt)*) => (
        $crate::korio_debug!(
            target: module_path!(),
            parent: $parent,
            { $($k).+, $($attribute)*}
        )
    );
    (parent: $parent:expr, %$($k:ident).+, $($attribute:tt)*) => (
        $crate::korio_debug!(
            target: module_path!(),
            parent: $parent,
            { %$($k).+, $($attribute)*}
        )
    );
    (parent: $parent:expr, ?$($k:ident).+, $($attribute:tt)*) => (
        $crate::korio_debug!(
            target: module_path!(),
            parent: $parent,
            { ?$($k).+, $($attribute)*}
        )
    );
    (parent: $parent:expr, $($arg:tt)+ ) => (
        $crate::korio_debug!(target: module_path!(), parent: $parent, { $($arg)+ })
    );

    // ...
    ( { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_debug!(
            target: module_path!(),
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    ( { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_debug!(
            target: module_path!(),
            { message = format_args!($($arg)+), $($attributes)* }
        )
    );
    ($($k:ident).+ = $($attribute:tt)*) => (
        $crate::korio_debug!(
            target: module_path!(),
            { $($k).+ = $($attribute)*}
        )
    );
    ($($k:ident).+, $($attribute:tt)*) => (
        $crate::korio_debug!(
            target: module_path!(),
            { $($k).+, $($attribute)*}
        )
    );
    (?$($k:ident).+, $($attribute:tt)*) => (
        $crate::korio_debug!(
            target: module_path!(),
            { ?$($k).+, $($attribute)*}
        )
    );
    (%$($k:ident).+, $($attribute:tt)*) => (
        $crate::korio_debug!(
            target: module_path!(),
            { %$($k).+, $($attribute)*}
        )
    );
    (?$($k:ident).+) => (
        $crate::korio_debug!(?$($k).+,)
    );
    (%$($k:ident).+) => (
        $crate::korio_debug!(%$($k).+,)
    );
    ($($k:ident).+) => (
        $crate::korio_debug!($($k).+,)
    );
    ( $($arg:tt)+ ) => (
        $crate::korio_debug!(target: module_path!(), { $($arg)+ })
    );
}

/// Construct a new event at the info severity.
///
#[macro_export]
macro_rules! korio_info {
    // name / target / parent
    (name: $name:expr, target: $target:expr, parent: $parent:expr, { $($attributes:tt)* } )=> (
        $crate::korio_event!(name: $name, target: $target, parent: $parent, $crate::Severity::Info, { $($attributes)* })
    );
    (name: $name:expr, target: $target:expr, parent: $parent:expr, { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_info!(
            name: $name,
            target: $target,
            parent: $parent,
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    (name: $name:expr, target: $target:expr, parent: $parent:expr, $($k:ident).+ = $($attributes:tt)* ) => (
        $crate::korio_info!(name: $name, target: $target, parent: $parent, { $($k).+ = $($attributes)* })
    );
    (name: $name:expr, target: $target:expr, parent: $parent:expr, $($arg:tt)+) => (
        $crate::korio_info!(name: $name, target: $target, parent: $parent, { $($arg)+ })
    );

    // name / target
    (name: $name:expr, target: $target:expr, { $($attributes:tt)* } )=> (
        $crate::korio_event!(name: $name, target: $target, $crate::Severity::Info, { $($attributes)* })
    );
    (name: $name:expr, target: $target:expr, { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_info!(
            name: $name,
            target: $target,
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    (name: $name:expr, target: $target:expr, $($k:ident).+ = $($attributes:tt)* ) => (
        $crate::korio_info!(name: $name, target: $target, { $($k).+ = $($attributes)* })
    );
    (name: $name:expr, target: $target:expr, $($arg:tt)+) => (
        $crate::korio_info!(name: $name, target: $target, { $($arg)+ })
    );

    // target / parent
    (target: $target:expr, parent: $parent:expr, { $($attributes:tt)* } )=> (
        $crate::korio_event!(target: $target, parent: $parent, $crate::Severity::Info, { $($attributes)* })
    );
    (target: $target:expr, parent: $parent:expr, { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_info!(
            target: $target,
            parent: $parent,
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    (target: $target:expr, parent: $parent:expr, $($k:ident).+ = $($attributes:tt)* ) => (
        $crate::korio_info!(target: $target, parent: $parent, { $($k).+ = $($attributes)* })
    );
    (target: $target:expr, parent: $parent:expr, $($arg:tt)+) => (
        $crate::korio_info!(target: $target, parent: $parent, { $($arg)+ })
    );

    // name / parent
    (name: $name:expr, parent: $parent:expr, { $($attributes:tt)* } )=> (
        $crate::korio_event!(name: $name, parent: $parent, $crate::Severity::Info, { $($attributes)* })
    );
    (name: $name:expr, parent: $parent:expr, { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_info!(
            name: $name,
            parent: $parent,
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    (name: $name:expr, parent: $parent:expr, $($k:ident).+ = $($attributes:tt)* ) => (
        $crate::korio_info!(name: $name, parent: $parent, { $($k).+ = $($attributes)* })
    );
    (name: $name:expr, parent: $parent:expr, $($arg:tt)+) => (
        $crate::korio_info!(name: $name, parent: $parent, { $($arg)+ })
    );

    // name
    (name: $name:expr, { $($attributes:tt)* } )=> (
        $crate::korio_event!(name: $name, $crate::Severity::Info, { $($attributes)* })
    );
    (name: $name:expr, { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_info!(
            name: $name,
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    (name: $name:expr, $($k:ident).+ = $($attributes:tt)* ) => (
        $crate::korio_info!(name: $name, { $($k).+ = $($attributes)* })
    );
    (name: $name:expr, $($arg:tt)+ ) => (
        $crate::korio_info!(name: $name, { $($arg)+ })
    );

    // target
    (target: $target:expr, { $($attributes:tt)* } )=> (
        $crate::korio_event!(target: $target, $crate::Severity::Info, { $($attributes)* })
    );
    (target: $target:expr, { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_info!(
            target: $target,
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    (target: $target:expr, $($k:ident).+ = $($attributes:tt)* ) => (
        $crate::korio_info!(target: $target, { $($k).+ = $($attributes)* })
    );
    (target: $target:expr, $($arg:tt)+ ) => (
        $crate::korio_info!(target: $target, { $($arg)+ })
    );

    // parent
    (parent: $parent:expr, { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_info!(
            target: module_path!(),
            parent: $parent,
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    (parent: $parent:expr, $($k:ident).+ = $($attribute:tt)*) => (
        $crate::korio_info!(
            target: module_path!(),
            parent: $parent,
            { $($k).+ = $($attribute)*}
        )
    );
    (parent: $parent:expr, ?$($k:ident).+ = $($attribute:tt)*) => (
        $crate::korio_info!(
            target: module_path!(),
            parent: $parent,
            { ?$($k).+ = $($attribute)*}
        )
    );
    (parent: $parent:expr, %$($k:ident).+ = $($attribute:tt)*) => (
        $crate::korio_info!(
            target: module_path!(),
            parent: $parent,
            { %$($k).+ = $($attribute)*}
        )
    );
    (parent: $parent:expr, $($k:ident).+, $($attribute:tt)*) => (
        $crate::korio_info!(
            target: module_path!(),
            parent: $parent,
            { $($k).+, $($attribute)*}
        )
    );
    (parent: $parent:expr, %$($k:ident).+, $($attribute:tt)*) => (
        $crate::korio_info!(
            target: module_path!(),
            parent: $parent,
            { %$($k).+, $($attribute)*}
        )
    );
    (parent: $parent:expr, ?$($k:ident).+, $($attribute:tt)*) => (
        $crate::korio_info!(
            target: module_path!(),
            parent: $parent,
            { ?$($k).+, $($attribute)*}
        )
    );
    (parent: $parent:expr, $($arg:tt)+ ) => (
        $crate::korio_info!(target: module_path!(), parent: $parent, { $($arg)+ })
    );

    // ...
    ( { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_info!(
            target: module_path!(),
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    ( { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_info!(
            target: module_path!(),
            { message = format_args!($($arg)+), $($attributes)* }
        )
    );
    ($($k:ident).+ = $($attribute:tt)*) => (
        $crate::korio_info!(
            target: module_path!(),
            { $($k).+ = $($attribute)*}
        )
    );
    ($($k:ident).+, $($attribute:tt)*) => (
        $crate::korio_info!(
            target: module_path!(),
            { $($k).+, $($attribute)*}
        )
    );
    (?$($k:ident).+, $($attribute:tt)*) => (
        $crate::korio_info!(
            target: module_path!(),
            { ?$($k).+, $($attribute)*}
        )
    );
    (%$($k:ident).+, $($attribute:tt)*) => (
        $crate::korio_info!(
            target: module_path!(),
            { %$($k).+, $($attribute)*}
        )
    );
    (?$($k:ident).+) => (
        $crate::korio_info!(?$($k).+,)
    );
    (%$($k:ident).+) => (
        $crate::korio_info!(%$($k).+,)
    );
    ($($k:ident).+) => (
        $crate::korio_info!($($k).+,)
    );
    ( $($arg:tt)+ ) => (
        $crate::korio_info!(target: module_path!(), { $($arg)+ })
    );
}

/// Construct a new event at the warn severity.
///
#[macro_export]
macro_rules! korio_warn {
    // name / target / parent
    (name: $name:expr, target: $target:expr, parent: $parent:expr, { $($attributes:tt)* } )=> (
        $crate::korio_event!(name: $name, target: $target, parent: $parent, $crate::Severity::Warn, { $($attributes)* })
    );
    (name: $name:expr, target: $target:expr, parent: $parent:expr, { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_warn!(
            name: $name,
            target: $target,
            parent: $parent,
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    (name: $name:expr, target: $target:expr, parent: $parent:expr, $($k:ident).+ = $($attributes:tt)* ) => (
        $crate::korio_warn!(name: $name, target: $target, parent: $parent, { $($k).+ = $($attributes)* })
    );
    (name: $name:expr, target: $target:expr, parent: $parent:expr, $($arg:tt)+) => (
        $crate::korio_warn!(name: $name, target: $target, parent: $parent, { $($arg)+ })
    );

    // name / target
    (name: $name:expr, target: $target:expr, { $($attributes:tt)* } )=> (
        $crate::korio_event!(name: $name, target: $target, $crate::Severity::Warn, { $($attributes)* })
    );
    (name: $name:expr, target: $target:expr, { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_warn!(
            name: $name,
            target: $target,
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    (name: $name:expr, target: $target:expr, $($k:ident).+ = $($attributes:tt)* ) => (
        $crate::korio_warn!(name: $name, target: $target, { $($k).+ = $($attributes)* })
    );
    (name: $name:expr, target: $target:expr, $($arg:tt)+) => (
        $crate::korio_warn!(name: $name, target: $target, { $($arg)+ })
    );

    // target / parent
    (target: $target:expr, parent: $parent:expr, { $($attributes:tt)* } )=> (
        $crate::korio_event!(target: $target, parent: $parent, $crate::Severity::Warn, { $($attributes)* })
    );
    (target: $target:expr, parent: $parent:expr, { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_warn!(
            target: $target,
            parent: $parent,
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    (target: $target:expr, parent: $parent:expr, $($k:ident).+ = $($attributes:tt)* ) => (
        $crate::korio_warn!(target: $target, parent: $parent, { $($k).+ = $($attributes)* })
    );
    (target: $target:expr, parent: $parent:expr, $($arg:tt)+) => (
        $crate::korio_warn!(target: $target, parent: $parent, { $($arg)+ })
    );

    // name / parent
    (name: $name:expr, parent: $parent:expr, { $($attributes:tt)* } )=> (
        $crate::korio_event!(name: $name, parent: $parent, $crate::Severity::Warn, { $($attributes)* })
    );
    (name: $name:expr, parent: $parent:expr, { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_warn!(
            name: $name,
            parent: $parent,
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    (name: $name:expr, parent: $parent:expr, $($k:ident).+ = $($attributes:tt)* ) => (
        $crate::korio_warn!(name: $name, parent: $parent, { $($k).+ = $($attributes)* })
    );
    (name: $name:expr, parent: $parent:expr, $($arg:tt)+) => (
        $crate::korio_warn!(name: $name, parent: $parent, { $($arg)+ })
    );

    // name
    (name: $name:expr, { $($attributes:tt)* } )=> (
        $crate::korio_event!(name: $name, $crate::Severity::Warn, { $($attributes)* })
    );
    (name: $name:expr, { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_warn!(
            name: $name,
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    (name: $name:expr, $($k:ident).+ = $($attributes:tt)* ) => (
        $crate::korio_warn!(name: $name, { $($k).+ = $($attributes)* })
    );
    (name: $name:expr, $($arg:tt)+ ) => (
        $crate::korio_warn!(name: $name, { $($arg)+ })
    );

    // target
    (target: $target:expr, { $($attributes:tt)* } )=> (
        $crate::korio_event!(target: $target, $crate::Severity::Warn, { $($attributes)* })
    );
    (target: $target:expr, { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_warn!(
            target: $target,
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    (target: $target:expr, $($k:ident).+ = $($attributes:tt)* ) => (
        $crate::korio_warn!(target: $target, { $($k).+ = $($attributes)* })
    );
    (target: $target:expr, $($arg:tt)+ ) => (
        $crate::korio_warn!(target: $target, { $($arg)+ })
    );

    // parent
    (parent: $parent:expr, { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_warn!(
            target: module_path!(),
            parent: $parent,
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    (parent: $parent:expr, $($k:ident).+ = $($attribute:tt)*) => (
        $crate::korio_warn!(
            target: module_path!(),
            parent: $parent,
            { $($k).+ = $($attribute)*}
        )
    );
    (parent: $parent:expr, ?$($k:ident).+ = $($attribute:tt)*) => (
        $crate::korio_warn!(
            target: module_path!(),
            parent: $parent,
            { ?$($k).+ = $($attribute)*}
        )
    );
    (parent: $parent:expr, %$($k:ident).+ = $($attribute:tt)*) => (
        $crate::korio_warn!(
            target: module_path!(),
            parent: $parent,
            { %$($k).+ = $($attribute)*}
        )
    );
    (parent: $parent:expr, $($k:ident).+, $($attribute:tt)*) => (
        $crate::korio_warn!(
            target: module_path!(),
            parent: $parent,
            { $($k).+, $($attribute)*}
        )
    );
    (parent: $parent:expr, %$($k:ident).+, $($attribute:tt)*) => (
        $crate::korio_warn!(
            target: module_path!(),
            parent: $parent,
            { %$($k).+, $($attribute)*}
        )
    );
    (parent: $parent:expr, ?$($k:ident).+, $($attribute:tt)*) => (
        $crate::korio_warn!(
            target: module_path!(),
            parent: $parent,
            { ?$($k).+, $($attribute)*}
        )
    );
    (parent: $parent:expr, $($arg:tt)+ ) => (
        $crate::korio_warn!(target: module_path!(), parent: $parent, { $($arg)+ })
    );

    // ...
    ( { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_warn!(
            target: module_path!(),
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    ( { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_warn!(
            target: module_path!(),
            { message = format_args!($($arg)+), $($attributes)* }
        )
    );
    ($($k:ident).+ = $($attribute:tt)*) => (
        $crate::korio_warn!(
            target: module_path!(),
            { $($k).+ = $($attribute)*}
        )
    );
    ($($k:ident).+, $($attribute:tt)*) => (
        $crate::korio_warn!(
            target: module_path!(),
            { $($k).+, $($attribute)*}
        )
    );
    (?$($k:ident).+, $($attribute:tt)*) => (
        $crate::korio_warn!(
            target: module_path!(),
            { ?$($k).+, $($attribute)*}
        )
    );
    (%$($k:ident).+, $($attribute:tt)*) => (
        $crate::korio_warn!(
            target: module_path!(),
            { %$($k).+, $($attribute)*}
        )
    );
    (?$($k:ident).+) => (
        $crate::korio_warn!(?$($k).+,)
    );
    (%$($k:ident).+) => (
        $crate::korio_warn!(%$($k).+,)
    );
    ($($k:ident).+) => (
        $crate::korio_warn!($($k).+,)
    );
    ( $($arg:tt)+ ) => (
        $crate::korio_warn!(target: module_path!(), { $($arg)+ })
    );
}

/// Construct a new event at the error severity.
///
#[macro_export]
macro_rules! korio_error {
    // name / target / parent
    (name: $name:expr, target: $target:expr, parent: $parent:expr, { $($attributes:tt)* } )=> (
        $crate::korio_event!(name: $name, target: $target, parent: $parent, $crate::Severity::Error, { $($attributes)* })
    );
    (name: $name:expr, target: $target:expr, parent: $parent:expr, { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_error!(
            name: $name,
            target: $target,
            parent: $parent,
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    (name: $name:expr, target: $target:expr, parent: $parent:expr, $($k:ident).+ = $($attributes:tt)* ) => (
        $crate::korio_error!(name: $name, target: $target, parent: $parent, { $($k).+ = $($attributes)* })
    );
    (name: $name:expr, target: $target:expr, parent: $parent:expr, $($arg:tt)+) => (
        $crate::korio_error!(name: $name, target: $target, parent: $parent, { $($arg)+ })
    );

    // name / target
    (name: $name:expr, target: $target:expr, { $($attributes:tt)* } )=> (
        $crate::korio_event!(name: $name, target: $target, $crate::Severity::Error, { $($attributes)* })
    );
    (name: $name:expr, target: $target:expr, { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_error!(
            name: $name,
            target: $target,
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    (name: $name:expr, target: $target:expr, $($k:ident).+ = $($attributes:tt)* ) => (
        $crate::korio_error!(name: $name, target: $target, { $($k).+ = $($attributes)* })
    );
    (name: $name:expr, target: $target:expr, $($arg:tt)+) => (
        $crate::korio_error!(name: $name, target: $target, { $($arg)+ })
    );

    // target / parent
    (target: $target:expr, parent: $parent:expr, { $($attributes:tt)* } )=> (
        $crate::korio_event!(target: $target, parent: $parent, $crate::Severity::Error, { $($attributes)* })
    );
    (target: $target:expr, parent: $parent:expr, { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_error!(
            target: $target,
            parent: $parent,
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    (target: $target:expr, parent: $parent:expr, $($k:ident).+ = $($attributes:tt)* ) => (
        $crate::korio_error!(target: $target, parent: $parent, { $($k).+ = $($attributes)* })
    );
    (target: $target:expr, parent: $parent:expr, $($arg:tt)+) => (
        $crate::korio_error!(target: $target, parent: $parent, { $($arg)+ })
    );

    // name / parent
    (name: $name:expr, parent: $parent:expr, { $($attributes:tt)* } )=> (
        $crate::korio_event!(name: $name, parent: $parent, $crate::Severity::Error, { $($attributes)* })
    );
    (name: $name:expr, parent: $parent:expr, { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_error!(
            name: $name,
            parent: $parent,
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    (name: $name:expr, parent: $parent:expr, $($k:ident).+ = $($attributes:tt)* ) => (
        $crate::korio_error!(name: $name, parent: $parent, { $($k).+ = $($attributes)* })
    );
    (name: $name:expr, parent: $parent:expr, $($arg:tt)+) => (
        $crate::korio_error!(name: $name, parent: $parent, { $($arg)+ })
    );

    // name
    (name: $name:expr, { $($attributes:tt)* } )=> (
        $crate::korio_event!(name: $name, $crate::Severity::Error, { $($attributes)* })
    );
    (name: $name:expr, { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_error!(
            name: $name,
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    (name: $name:expr, $($k:ident).+ = $($attributes:tt)* ) => (
        $crate::korio_error!(name: $name, { $($k).+ = $($attributes)* })
    );
    (name: $name:expr, $($arg:tt)+ ) => (
        $crate::korio_error!(name: $name, { $($arg)+ })
    );

    // target
    (target: $target:expr, { $($attributes:tt)* } )=> (
        $crate::korio_event!(target: $target, $crate::Severity::Error, { $($attributes)* })
    );
    (target: $target:expr, { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_error!(
            target: $target,
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    (target: $target:expr, $($k:ident).+ = $($attributes:tt)* ) => (
        $crate::korio_error!(target: $target, { $($k).+ = $($attributes)* })
    );
    (target: $target:expr, $($arg:tt)+ ) => (
        $crate::korio_error!(target: $target, { $($arg)+ })
    );

    // parent
    (parent: $parent:expr, { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_error!(
            target: module_path!(),
            parent: $parent,
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    (parent: $parent:expr, $($k:ident).+ = $($attribute:tt)*) => (
        $crate::korio_error!(
            target: module_path!(),
            parent: $parent,
            { $($k).+ = $($attribute)*}
        )
    );
    (parent: $parent:expr, ?$($k:ident).+ = $($attribute:tt)*) => (
        $crate::korio_error!(
            target: module_path!(),
            parent: $parent,
            { ?$($k).+ = $($attribute)*}
        )
    );
    (parent: $parent:expr, %$($k:ident).+ = $($attribute:tt)*) => (
        $crate::korio_error!(
            target: module_path!(),
            parent: $parent,
            { %$($k).+ = $($attribute)*}
        )
    );
    (parent: $parent:expr, $($k:ident).+, $($attribute:tt)*) => (
        $crate::korio_error!(
            target: module_path!(),
            parent: $parent,
            { $($k).+, $($attribute)*}
        )
    );
    (parent: $parent:expr, %$($k:ident).+, $($attribute:tt)*) => (
        $crate::korio_error!(
            target: module_path!(),
            parent: $parent,
            { %$($k).+, $($attribute)*}
        )
    );
    (parent: $parent:expr, ?$($k:ident).+, $($attribute:tt)*) => (
        $crate::korio_error!(
            target: module_path!(),
            parent: $parent,
            { ?$($k).+, $($attribute)*}
        )
    );
    (parent: $parent:expr, $($arg:tt)+ ) => (
        $crate::korio_error!(target: module_path!(), parent: $parent, { $($arg)+ })
    );

    // ...
    ( { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_error!(
            target: module_path!(),
            { message = $crate::__macro_support::format_args!($($arg)+), $($attributes)* }
        )
    );
    ( { $($attributes:tt)* }, $($arg:tt)+ ) => (
        $crate::korio_error!(
            target: module_path!(),
            { message = format_args!($($arg)+), $($attributes)* }
        )
    );
    ($($k:ident).+ = $($attribute:tt)*) => (
        $crate::korio_error!(
            target: module_path!(),
            { $($k).+ = $($attribute)*}
        )
    );
    ($($k:ident).+, $($attribute:tt)*) => (
        $crate::korio_error!(
            target: module_path!(),
            { $($k).+, $($attribute)*}
        )
    );
    (?$($k:ident).+, $($attribute:tt)*) => (
        $crate::korio_error!(
            target: module_path!(),
            { ?$($k).+, $($attribute)*}
        )
    );
    (%$($k:ident).+, $($attribute:tt)*) => (
        $crate::korio_error!(
            target: module_path!(),
            { %$($k).+, $($attribute)*}
        )
    );
    (?$($k:ident).+) => (
        $crate::korio_error!(?$($k).+,)
    );
    (%$($k:ident).+) => (
        $crate::korio_error!(%$($k).+,)
    );
    ($($k:ident).+) => (
        $crate::korio_error!($($k).+,)
    );
    ( $($arg:tt)+ ) => (
        $crate::korio_error!(target: module_path!(), { $($arg)+ })
    );
}
