mod parse_data;

use parse_data::get_json_contents;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json_data = get_json_contents()?;


    todo!()
}
