use moon_process::{output_to_string, Command};
use once_map::OnceMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct ProcessCache {
    /// Output cache of all executed commands.
    cache: OnceMap<String, String>,

    /// Binary/command to run.
    pub bin: String,

    /// Root of the moon workspace, and where to run commands.
    pub root: PathBuf,
}

impl ProcessCache {
    pub fn new(bin: &str, root: &Path) -> Self {
        Self {
            cache: OnceMap::new(),
            bin: bin.to_string(),
            root: root.to_path_buf(),
        }
    }

    pub fn create_command<I, A>(&self, args: I) -> Command
    where
        I: IntoIterator<Item = A>,
        A: AsRef<OsStr>,
    {
        let mut command = Command::new(&self.bin);
        command.args(args);
        // Run from workspace root instead of git root so that we can avoid
        // prefixing all file paths to ensure everything is relative and accurate.
        command.cwd(&self.root);
        command
    }

    pub async fn run<I, A>(&self, args: I, trim: bool) -> miette::Result<&str>
    where
        I: IntoIterator<Item = A>,
        A: AsRef<OsStr>,
    {
        self.run_command(self.create_command(args), trim).await
    }

    pub async fn run_with_formatter<I, A>(
        &self,
        args: I,
        trim: bool,
        format: impl FnOnce(String) -> String,
    ) -> miette::Result<&str>
    where
        I: IntoIterator<Item = A>,
        A: AsRef<OsStr>,
    {
        self.run_command_with_formatter(self.create_command(args), trim, format)
            .await
    }

    pub async fn run_command(&self, command: Command, trim: bool) -> miette::Result<&str> {
        self.run_command_with_formatter(command, trim, |s| s).await
    }

    pub async fn run_command_with_formatter(
        &self,
        command: Command,
        trim: bool,
        format: impl FnOnce(String) -> String,
    ) -> miette::Result<&str> {
        let mut executor = command.create_async();
        let cache_key = executor.inspector.get_cache_key();

        // Execute and insert output into the cache if not already present
        if !self.cache.contains_key(&cache_key) {
            let output = executor.exec_capture_output().await?;

            self.cache.insert(cache_key.clone(), |_| {
                format(output_to_string(&output.stdout))
            });
        }

        let output = self.cache.get(&cache_key).unwrap();

        Ok(if trim { output.trim() } else { output })
    }
}
