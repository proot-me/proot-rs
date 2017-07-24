
macro_rules! get {
    ($header:expr, $field:ident, $result_type:ty) => {
        $header.apply(
            |header32| Ok(header32.$field as $result_type),
            |header64| Ok(header64.$field as $result_type)
        )
    };
    ($header:expr, $field:ident) => {
        $header.apply(
            |header32| Ok(header32.$field),
            |header64| Ok(header64.$field)
        )
    };
}

macro_rules! apply {
    ($header:expr, $func:expr)    => {
            $header.apply(
                $func,
                $func
            )
     };
}