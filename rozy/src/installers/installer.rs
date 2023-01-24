use anyhow::Result;

pub trait Installer {
    fn install(&self, to_dir: &std::path::Path) -> Result<()>;
    fn describe(&self) -> String;
}
