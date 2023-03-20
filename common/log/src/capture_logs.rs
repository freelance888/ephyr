use async_trait::async_trait;
use std::{io, process::Output};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Child,
};
use tracing::{Instrument, Span};

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
#[async_trait]
pub trait ChildCapture {
    /// Redirect logs from stdout and stderr of [Child] process
    /// to `tracing`. Where `parser` is user defined function to parse log line.
    async fn capture_logs_and_wait_for_output<F>(
        mut self,
        span: Span,
        parser: F,
    ) -> io::Result<Output>
    where
        Self: Sized,
        F: Fn(&str) -> ParsedMsg<'_> + Send + 'static;
}

/// Pass [ParsedMsg.message] to [tracing] based on [ParsedMsg.level].
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

#[async_trait]
impl ChildCapture for Child {
    async fn capture_logs_and_wait_for_output<F>(
        mut self,
        span: Span,
        parser: F,
    ) -> io::Result<Output>
    where
        F: Fn(&str) -> ParsedMsg<'_> + Send + 'static,
    {
        let out_buff = self.stdout.take().map(BufReader::new).unwrap();
        let err_buff = self.stderr.take().map(BufReader::new).unwrap();

        let process_id = self.id();

        let mut out_lines = out_buff.lines();
        let mut err_lines = err_buff.lines();

        let mut out_done = false;
        let mut err_done = false;

        let capture_task = tokio::spawn(
            async move {
                loop {
                    if out_done && err_done {
                        break;
                    }

                    tokio::select! {
                        out_line = out_lines.next_line(), if !out_done => {
                            if let Some(out_line) = out_line.ok().flatten() {
                                capture_line(
                                    process_id,
                                    &span,
                                    parser(&out_line),
                                );
                            } else {
                                out_done = true;
                            }
                        }
                        err_line = err_lines.next_line(), if !err_done => {
                            if let Some(err_line) = err_line.ok().flatten() {
                                capture_line(
                                    process_id,
                                    &span,
                                    parser(&err_line),
                                );
                            } else {
                                err_done = true;
                            }
                        }
                    }
                }
            }
            .in_current_span(),
        );

        let out = self.wait_with_output().await;
        capture_task.abort();
        out
    }
}
