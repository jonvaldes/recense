pub trait Context<T, E> {
    fn with_context<C, F>(self, f: F) -> Result<T, failure::Error>
    where
        C: std::fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C;
}

impl<T, E> Context<T,E> for ::std::result::Result<T, E>
where
    E: failure::Fail,
{
    fn with_context<C, F>(self, context_func: F) -> ::std::result::Result<T, failure::Error>
    where
        C: std::fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        match self {
            Ok(x) => Ok(x),
            Err(err) => {
                #[derive(Fail, Debug)]
                #[fail(display = "Error: \"{}\". With context: \"{}\"", inner, context)]
                struct InnerFail {
                    #[cause]
                    inner: failure::Error,
                    context: String,
                }

                Err(failure::Error::from(InnerFail {
                    inner: failure::Error::from(err),
                    context: context_func().to_string(),
                }))
            }
        }
    }
}
