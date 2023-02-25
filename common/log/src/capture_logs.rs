use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Child,
};
use tracing::{instrument, Instrument, Span};

const TARGET: &str = "from_proc";

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
    fn capture_logs<F>(&mut self, span: Span, parser: F)
    where
        Self: Sized,
        F: Fn(&str) -> ParsedMsg<'_> + Send + 'static;
}

/// Pass [ParsedMsg.message] to [tracing] based on [ParsedMsg.level].
#[instrument(parent = span, target="from_proc", skip_all)]
fn capture_line(pid: Option<u32>, span: &Span, parsed_msg: ParsedMsg) {
    let ParsedMsg { level, message } = parsed_msg;
    // TODO: convert to HashMap when tracing `valuable` stabilizes.
    match level.to_lowercase().as_str() {
        "error" => {
            tracing::error!(target: TARGET, parent: span, message, pid);
        }
        "info" => {
            tracing::info!(target: TARGET, parent: span, message, pid);
        }
        "debug" | "verbose" => {
            tracing::debug!(target: TARGET, parent: span, message, pid);
        }
        _ => tracing::trace!(target: TARGET, parent: span, message, pid),
    };
}

impl ChildCapture for Child {
    #[instrument(parent=&span, skip_all)]
    fn capture_logs<F>(&mut self, span: Span, parser: F)
    where
        F: Fn(&str) -> ParsedMsg<'_> + Send + 'static,
    {
        let out_buff = self.stdout.take().map(BufReader::new).unwrap();
        let err_buff = self.stderr.take().map(BufReader::new).unwrap();

        let process_id = self.id();

        drop(tokio::spawn(
            async move {
                let mut out_lines = out_buff.lines();
                let mut err_lines = err_buff.lines();
                loop {
                    if let Some(out_line) =
                        out_lines.next_line().await.ok().flatten()
                    {
                        capture_line(process_id, &span, parser(&out_line));
                    }
                    if let Some(err_line) =
                        err_lines.next_line().await.ok().flatten()
                    {
                        capture_line(process_id, &span, parser(&err_line));
                    }
                }
            }
            .in_current_span(),
        ));
    }
}
