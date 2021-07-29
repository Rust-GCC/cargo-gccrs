//! Fetch extra arguments from the environment

/// All kinds of availabe environment arguments
pub enum EnvArgs {
    /// Arguments used when creating a static library
    Ar,
    /// Arguments given to the compiler during compilation
    Gcc,
}

impl EnvArgs {
    /// Get the environment variable key associated with an instance of EnvArgs
    fn as_key(&self) -> &'static str {
        match self {
            EnvArgs::Ar => "AR_EXTRA_ARGS",
            EnvArgs::Gcc => "GCCRS_EXTRA_ARGS",
        }
    }

    /// Fetch the extra arguments given by the user for a specific environment string
    pub fn as_args(&self) -> Option<Vec<String>> {
        std::env::var(self.as_key())
            .map(|s| s.split(' ').map(|arg| arg.to_owned()).collect())
            .ok()
    }
}
