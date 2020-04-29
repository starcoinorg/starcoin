#[macro_export]
macro_rules! register_uint_gauge_vec {
    ($OPTS:expr, $LABELS_NAMES:expr) => {{
        __register_gauge_vec!(UIntGaugeVec, $OPTS, $LABELS_NAMES)
    }};

    ($NAME:expr, $HELP:expr, $LABELS_NAMES:expr) => {{
        register_uint_gauge_vec!(opts!($NAME, $HELP), $LABELS_NAMES)
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! register_uint_gauge {
    ($OPTS:expr) => {{
        let gauge = $crate::UIntGauge::with_opts($OPTS).unwrap();
        $crate::register(Box::new(gauge.clone())).map(|_| gauge)
    }};
    ($NAME:expr, $HELP:expr) => {{
        register_uint_gauge!($crate::opts!($NAME, $HELP))
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! __register_gauge_vec {
    ($TYPE:ident, $OPTS:expr, $LABELS_NAMES:expr) => {{
        let gauge_vec = $crate::$TYPE::new($OPTS, $LABELS_NAMES).unwrap();
        prometheus::register(Box::new(gauge_vec.clone())).map(|_| gauge_vec)
    }};
}
