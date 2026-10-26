//! Test-only native Git fixture ownership, independent of the foreground executor.

#[cfg(not(windows))]
pub(super) struct NativeProcess(tokio::process::Child);

#[cfg(not(windows))]
impl NativeProcess {
    pub(super) async fn spawn(
        command: &mut tokio::process::Command,
        _diagnostics: &tempfile::NamedTempFile,
    ) -> Result<Self, String> {
        command.spawn().map(Self).map_err(|e| e.to_string())
    }

    pub(super) fn is_running(&mut self) -> Result<bool, String> {
        self.0
            .try_wait()
            .map(|s| s.is_none())
            .map_err(|e| e.to_string())
    }

    pub(super) async fn wait(&mut self) -> Result<bool, String> {
        self.0
            .wait()
            .await
            .map(|s| s.success())
            .map_err(|e| e.to_string())
    }

    pub(super) async fn wait_with_deadline(
        &mut self,
        deadline: std::time::Duration,
    ) -> Result<bool, String> {
        match tokio::time::timeout(deadline, self.wait()).await {
            Ok(outcome) => outcome,
            Err(_) => {
                self.stop().await.map_err(|error| {
                    format!("fixture deadline elapsed; direct-child cleanup unconfirmed: {error}")
                })?;
                Err("fixture deadline elapsed; direct-child cleanup confirmed; descendants not supervised on this platform".into())
            }
        }
    }

    pub(super) async fn stop(&mut self) -> Result<(), String> {
        self.0.kill().await.map_err(|e| e.to_string())?;
        self.0.wait().await.map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[cfg(windows)]
pub(super) struct NativeProcess {
    // The event pump never retains this control. Dropping the fixture, including
    // during panic, closes the kill-on-close Job instead of only killing git.exe.
    control: std::sync::Arc<dyn talos_core::background_job::BackgroundProcessControl>,
    done: tokio::sync::oneshot::Receiver<Result<bool, String>>,
    outcome: Option<Result<bool, String>>,
}

#[cfg(windows)]
impl NativeProcess {
    pub(super) async fn spawn(
        command: &mut tokio::process::Command,
        diagnostics: &tempfile::NamedTempFile,
    ) -> Result<Self, String> {
        use talos_core::background_job::{BackgroundJobLauncher, BackgroundProcessEvent};
        let command = command.as_std();
        let program = command
            .get_program()
            .to_str()
            .ok_or("non-Unicode fixture program")?;
        let args = command
            .get_args()
            .map(|arg| {
                arg.to_str()
                    .map(str::to_owned)
                    .ok_or("non-Unicode fixture argument")
            })
            .collect::<Result<Vec<_>, _>>()?;
        let env = command
            .get_envs()
            .map(|(key, value)| {
                Ok((
                    key.to_str()
                        .ok_or("non-Unicode fixture environment key")?
                        .to_owned(),
                    value
                        .ok_or("fixture environment removal unsupported")?
                        .to_str()
                        .ok_or("non-Unicode fixture environment value")?
                        .to_owned(),
                ))
            })
            .collect::<Result<std::collections::BTreeMap<_, _>, String>>()?;
        let cwd = command
            .get_current_dir()
            .map(std::path::Path::to_owned)
            .map(Ok)
            .unwrap_or_else(std::env::current_dir)
            .map_err(|e| e.to_string())?;
        let mut log = diagnostics.reopen().map_err(|e| e.to_string())?;
        let launched = Box::new(crate::process_boundary::WindowsBackgroundLauncher::new(
            program.to_owned(),
            args,
            cwd,
            env,
        ))
        .launch()
        .await?;
        let control = launched.control;
        let mut events = launched.events;
        let (sender, done) = tokio::sync::oneshot::channel();
        tokio::spawn(async move {
            use std::io::Write;
            let outcome = loop {
                match events.recv().await {
                    Some(BackgroundProcessEvent::Output(chunk)) => {
                        if let Err(error) = log.write_all(&chunk.bytes) {
                            break Err(error.to_string());
                        }
                    }
                    Some(BackgroundProcessEvent::Exited(exit)) => break Ok(exit.success),
                    Some(BackgroundProcessEvent::SupervisionFailed(error)) => break Err(error),
                    None => break Err("fixture event stream closed without exit".into()),
                }
            };
            let _ = sender.send(outcome);
        });
        Ok(Self {
            control,
            done,
            outcome: None,
        })
    }

    pub(super) fn is_running(&mut self) -> Result<bool, String> {
        if let Some(outcome) = &self.outcome {
            return match outcome {
                Ok(_) => Ok(false),
                Err(error) => Err(error.clone()),
            };
        }
        match self.done.try_recv() {
            Err(tokio::sync::oneshot::error::TryRecvError::Empty) => Ok(true),
            Ok(outcome) => {
                self.outcome = Some(outcome.clone());
                outcome.map(|_| false)
            }
            Err(error) => Err(error.to_string()),
        }
    }

    pub(super) async fn wait(&mut self) -> Result<bool, String> {
        let outcome = if let Some(outcome) = &self.outcome {
            outcome.clone()
        } else {
            let outcome = (&mut self.done).await.map_err(|e| e.to_string())?;
            self.outcome = Some(outcome.clone());
            outcome
        };
        self.control.terminate().await?;
        outcome
    }

    pub(super) async fn wait_with_deadline(
        &mut self,
        deadline: std::time::Duration,
    ) -> Result<bool, String> {
        match tokio::time::timeout(deadline, self.wait()).await {
            Ok(outcome) => outcome,
            Err(_) => {
                self.stop().await.map_err(|error| {
                    format!("fixture deadline elapsed; process-tree cleanup unconfirmed: {error}")
                })?;
                Err("fixture deadline elapsed; process-tree cleanup confirmed".into())
            }
        }
    }

    pub(super) async fn stop(&mut self) -> Result<(), String> {
        self.control.terminate().await?;
        tokio::time::timeout(std::time::Duration::from_secs(5), self.wait())
            .await
            .map_err(|_| "fixture event drain deadline exceeded".to_owned())??;
        Ok(())
    }
}
