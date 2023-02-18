use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Child,
};

/// The user defined `parser` function should return this struct.
#[derive(Debug, Copy, Clone)]
pub struct ParsedMsg<'a> {
    /// Log line message
    pub message: &'a str,
    /// Log line level
    pub level: &'a str,
}

/// Add option to capture logs from child process.
pub trait ChildCapture {
    /// Redirect logs from stdout and stderr of [Child] process
    /// to `tracing`. Where `parser` is user defined function to parse log line.
    fn capture_logs<F>(&mut self, parser: F)
    where
        Self: Sized,
        F: Fn(&str) -> ParsedMsg<'_> + Send + 'static;
}

/// Pass [ParsedMsg.message] to [tracing] based on [ParsedMsg.level].
fn capture_line(parsed_msg: ParsedMsg) {
    let ParsedMsg { level, message } = parsed_msg;
    match level.to_lowercase().as_str() {
        "error" => tracing::error!(message),
        "info" => tracing::info!(message),
        "debug" | "verbose" => tracing::debug!(message),
        _ => tracing::trace!(message),
    };
}

impl ChildCapture for Child {
    fn capture_logs<F>(&mut self, parser: F)
    where
        F: Fn(&str) -> ParsedMsg<'_> + Send + 'static,
    {
        let out_buff = self.stdout.take().map(BufReader::new).unwrap();
        let err_buff = self.stderr.take().map(BufReader::new).unwrap();

        drop(tokio::spawn(async move {
            let mut out_lines = out_buff.lines();
            let mut err_lines = err_buff.lines();
            loop {
                if let Some(out_line) =
                    out_lines.next_line().await.ok().flatten()
                {
                    capture_line(parser(&out_line));
                }
                if let Some(err_line) =
                    err_lines.next_line().await.ok().flatten()
                {
                    capture_line(parser(&err_line));
                }
            }
        }));
    }
}
