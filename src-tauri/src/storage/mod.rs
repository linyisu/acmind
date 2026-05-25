use std::path::PathBuf;

use crate::error::AppError;

#[derive(Clone)]
pub struct Storage {
    base_path: PathBuf,
}

impl Storage {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    /// Get the base data directory path.
    pub fn base_dir(&self) -> &PathBuf {
        &self.base_path
    }

    /// Get the problem directory path.
    fn problem_dir(&self, problem_id: &str) -> PathBuf {
        self.base_path.join("problems").join(&problem_id[..8]) // Use first 8 chars of UUID as subdir
    }

    /// Save a problem's statement as markdown.
    pub fn save_statement(&self, problem_id: &str, content: &str) -> Result<String, AppError> {
        let dir = self.problem_dir(problem_id);
        std::fs::create_dir_all(&dir)?;
        let path = dir.join("statement.md");
        std::fs::write(&path, content)?;
        Ok(path.to_string_lossy().to_string())
    }

    /// Read a file and return its content.
    pub fn read_file(&self, path: &str) -> Result<String, AppError> {
        Ok(std::fs::read_to_string(path)?)
    }

    /// Write a file.
    #[allow(dead_code)]
    pub fn write_file(&self, path: &str, content: &str) -> Result<(), AppError> {
        if let Some(parent) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Save a submission code file.
    pub fn save_submission(
        &self,
        problem_id: &str,
        submission_id: &str,
        status: &str,
        language: &str,
        code: &str,
    ) -> Result<String, AppError> {
        let lower_lang = language.to_lowercase();
        let ext = match lower_lang.as_str() {
            "c++" | "cpp" => "cpp",
            "python" | "py" => "py",
            "java" => "java",
            "rust" | "rs" => "rs",
            other => other,
        };

        let dir = self.problem_dir(problem_id).join("submissions");
        std::fs::create_dir_all(&dir)?;

        let filename = format!("{}_{}.{}", &submission_id[..8], status.to_lowercase(), ext);
        let path = dir.join(&filename);
        std::fs::write(&path, code)?;

        Ok(path.to_string_lossy().to_string())
    }

    /// Delete a problem directory and all its files.
    pub fn delete_problem_dir(&self, problem_id: &str) -> Result<(), AppError> {
        let dir = self.problem_dir(problem_id);
        if dir.exists() {
            std::fs::remove_dir_all(&dir)?;
        }
        Ok(())
    }

    /// Delete a specific file.
    pub fn delete_file(&self, path: &str) -> Result<(), AppError> {
        let p = std::path::Path::new(path);
        if p.exists() {
            std::fs::remove_file(p)?;
        }
        Ok(())
    }

    /// Read a submission's code text from its file path.
    pub fn read_code(&self, path: &str) -> Result<String, AppError> {
        self.read_file(path)
    }
}
