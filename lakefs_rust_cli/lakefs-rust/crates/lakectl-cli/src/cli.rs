use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "lakectl")]
#[command(about = "A Rust CLI for lakeFS")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    
    /// Config file path
    #[arg(global = true, short, long, env = "LAKECTL_CONFIG_FILE")]
    pub config: Option<String>,
    
    /// Enable verbose output
    #[arg(global = true, short, long)]
    pub verbose: bool,
    
    /// Disable color output
    #[arg(global = true, long)]
    pub no_color: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Manage repositories
    Repo {
        #[command(subcommand)]
        command: RepoCommands,
    },
    
    /// Manage branches
    Branch {
        #[command(subcommand)]
        command: BranchCommands,
    },
    
    /// Create commits
    Commit {
        /// Repository/branch URI
        #[arg(value_name = "BRANCH_URI")]
        branch: String,
        
        /// Commit message
        #[arg(short, long)]
        message: String,
        
        /// Allow empty commit
        #[arg(long)]
        allow_empty: bool,
    },
    
    /// View commit logs
    Log {
        /// Repository/branch URI
        #[arg(value_name = "BRANCH_URI")]
        branch: String,
        
        /// Number of commits to show
        #[arg(short, long, default_value = "10")]
        amount: usize,
    },
    
    /// File system operations
    Fs {
        #[command(subcommand)]
        command: FsCommands,
    },
    
    /// Show differences
    Diff {
        /// Left reference
        #[arg(value_name = "LEFT_REF")]
        left: String,
        
        /// Right reference
        #[arg(value_name = "RIGHT_REF")]
        right: Option<String>,
    },
    
    /// Merge branches
    Merge {
        /// Source reference
        #[arg(value_name = "SOURCE_REF")]
        source: String,
        
        /// Destination branch
        #[arg(value_name = "DEST_BRANCH")]
        destination: String,
        
        /// Merge strategy
        #[arg(long)]
        strategy: Option<String>,
    },
    
    /// Local directory operations
    Local {
        #[command(subcommand)]
        command: LocalCommands,
    },
}

#[derive(Subcommand)]
pub enum RepoCommands {
    /// Create a new repository
    Create {
        /// Repository name
        #[arg(value_name = "REPOSITORY")]
        name: String,
        
        /// Storage namespace
        #[arg(value_name = "STORAGE_NAMESPACE")]
        storage_namespace: String,
        
        /// Default branch name
        #[arg(long, default_value = "main")]
        default_branch: String,
    },
    
    /// List repositories
    List {
        /// Show only this many results
        #[arg(long)]
        amount: Option<usize>,
        
        /// Start after this value
        #[arg(long)]
        after: Option<String>,
    },
    
    /// Delete a repository
    Delete {
        /// Repository name
        #[arg(value_name = "REPOSITORY")]
        name: String,
        
        /// Skip confirmation
        #[arg(short, long)]
        yes: bool,
    },
}

#[derive(Subcommand)]
pub enum BranchCommands {
    /// Create a new branch
    Create {
        /// Branch URI
        #[arg(value_name = "BRANCH_URI")]
        uri: String,
        
        /// Source branch/commit
        #[arg(short, long)]
        source: String,
    },
    
    /// List branches
    List {
        /// Repository URI
        #[arg(value_name = "REPOSITORY_URI")]
        repository: String,
        
        /// Show only this many results
        #[arg(long)]
        amount: Option<usize>,
    },
    
    /// Delete a branch
    Delete {
        /// Branch URI
        #[arg(value_name = "BRANCH_URI")]
        uri: String,
        
        /// Skip confirmation
        #[arg(short, long)]
        yes: bool,
    },
    
    /// Show branch information
    Show {
        /// Branch URI
        #[arg(value_name = "BRANCH_URI")]
        uri: String,
    },
}

#[derive(Subcommand)]
pub enum FsCommands {
    /// List directory contents
    Ls {
        /// Path URI
        #[arg(value_name = "PATH_URI")]
        path: String,
        
        /// List recursively
        #[arg(short, long)]
        recursive: bool,
    },
    
    /// Download object
    Download {
        /// Source path URI
        #[arg(value_name = "SOURCE_URI")]
        source: String,
        
        /// Destination path
        #[arg(value_name = "DEST_PATH")]
        destination: Option<String>,
        
        /// Download recursively
        #[arg(short, long)]
        recursive: bool,
        
        /// Number of parallel downloads
        #[arg(short, long, default_value = "10")]
        parallelism: usize,
    },
    
    /// Upload object
    Upload {
        /// Source file/directory
        #[arg(value_name = "SOURCE_PATH")]
        source: String,
        
        /// Destination URI
        #[arg(value_name = "DEST_URI")]
        destination: String,
        
        /// Upload recursively
        #[arg(short, long)]
        recursive: bool,
        
        /// Number of parallel uploads
        #[arg(short, long, default_value = "10")]
        parallelism: usize,
    },
    
    /// Remove object
    Rm {
        /// Path URI
        #[arg(value_name = "PATH_URI")]
        path: String,
        
        /// Remove recursively
        #[arg(short, long)]
        recursive: bool,
    },
    
    /// Show object metadata
    Stat {
        /// Path URI
        #[arg(value_name = "PATH_URI")]
        path: String,
    },
}

#[derive(Subcommand)]
pub enum LocalCommands {
    /// Initialize local directory
    Init {
        /// Remote URI
        #[arg(value_name = "REMOTE_URI")]
        remote: String,
        
        /// Local directory
        #[arg(value_name = "LOCAL_PATH", default_value = ".")]
        path: String,
    },
    
    /// Clone repository to local directory
    Clone {
        /// Remote URI
        #[arg(value_name = "REMOTE_URI")]
        remote: String,
        
        /// Local directory
        #[arg(value_name = "LOCAL_PATH")]
        path: Option<String>,
    },
    
    /// Show local status
    Status {
        /// Local directory
        #[arg(value_name = "LOCAL_PATH", default_value = ".")]
        path: String,
    },
    
    /// Pull changes from remote
    Pull {
        /// Local directory
        #[arg(value_name = "LOCAL_PATH", default_value = ".")]
        path: String,
        
        /// Force pull (overwrite local changes)
        #[arg(long)]
        force: bool,
    },
    
    /// Commit and push local changes
    Commit {
        /// Local directory
        #[arg(value_name = "LOCAL_PATH", default_value = ".")]
        path: String,
        
        /// Commit message
        #[arg(short, long)]
        message: String,
    },
}
