struct X { }
struct X <'a> { }
struct X <'a, 'b> { }
struct X <'a, 'b> { }
pub struct ExtractedFile<'query> {
    file: Option<PathBuf>,
    file_type: String,
    matches: Vec<ExtractedMatch<'query>>,
}
pub fn new<E>(error: E) -> Self
        where
            E: StdError + Send + Sync + 'static,
        {
            let backtrace = backtrace_if_absent!(error);
            Error::from_std(error, backtrace)
        }
