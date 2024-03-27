mod utils;

type GenericResult = Result<(), Box<dyn std::error::Error>>;


fn main() -> GenericResult {
    Ok(())
}
