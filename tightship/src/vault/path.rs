use crate::failure::Failure;

pub fn run() -> Result<(), Failure> {
    println!("{}", super::folder()?.display());
    Ok(())
}
