use clap::{Arg, Command};
use ico::{IconDir};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct HotspotInfo {
    filename: String,
    hotspot_x: u16,
    hotspot_y: u16,
    width: u32,
    height: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("cur2png")
        .version("1.0")
        .author("Your Name")
        .about("Converts .cur files to .png with hotspot extraction")
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .value_name("INPUT_DIR")
                .help("Input directory containing .cur files")
                .required(true),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("OUTPUT_DIR")
                .help("Output directory for .png files")
                .required(true),
        )
        .arg(
            Arg::new("json")
                .short('j')
                .long("json")
                .value_name("JSON_FILE")
                .help("Output JSON file for hotspot data")
                .default_value("hotspots.json"),
        )
        .get_matches();

    let input_dir = matches.get_one::<String>("input").unwrap();
    let output_dir = matches.get_one::<String>("output").unwrap();
    let json_file = matches.get_one::<String>("json").unwrap();

    // Create output directory if it doesn't exist
    fs::create_dir_all(output_dir)?;

    let mut hotspot_data: HashMap<String, Value> = HashMap::new();
    let mut processed_count = 0;

    // Read input directory
    let input_path = Path::new(input_dir);
    if !input_path.exists() {
        eprintln!("Error: Input directory '{}' does not exist", input_dir);
        std::process::exit(1);
    }

    for entry in fs::read_dir(input_path)? {
        let entry = entry?;
        let path = entry.path();
        
        if let Some(extension) = path.extension() {
            if extension.to_str().unwrap_or("").to_lowercase() == "cur" {
                match process_cursor_file(&path, output_dir, &mut hotspot_data) {
                    Ok(info) => {
                        println!("Processed: {} -> {}", info.filename, 
                                format!("{}.png", Path::new(&info.filename).file_stem().unwrap().to_str().unwrap()));
                        processed_count += 1;
                    }
                    Err(e) => {
                        eprintln!("Error processing {}: {}", path.display(), e);
                    }
                }
            }
        }
    }

    // Write JSON file
    let json_output = json!(hotspot_data);
    fs::write(json_file, serde_json::to_string_pretty(&json_output)?)?;

    println!("\nConversion complete!");
    println!("Processed {} cursor files", processed_count);
    println!("PNG files saved to: {}", output_dir);
    println!("Hotspot data saved to: {}", json_file);

    Ok(())
}

fn process_cursor_file(
    cursor_path: &Path,
    output_dir: &str,
    hotspot_data: &mut HashMap<String, Value>,
) -> Result<HotspotInfo, Box<dyn std::error::Error>> {
    // Read the cursor file
    let file_data = fs::read(cursor_path)?;
    let icon_dir = IconDir::read(std::io::Cursor::new(file_data))?;

    let filename = cursor_path.file_name().unwrap().to_str().unwrap().to_string();
    let stem = cursor_path.file_stem().unwrap().to_str().unwrap();
    
    // Get the first (largest) cursor image
    let entry = icon_dir.entries().first()
        .ok_or("No cursor entries found in file")?;
    
    let image = entry.decode()?;
    
    // Extract hotspot information (cursor-specific)
    let (hotspot_x, hotspot_y) = match image.cursor_hotspot() {
        Some(hot) => hot,
        None => (0, 0),
    };

    // Convert IconImage to PNG
    let width = image.width();
    let height = image.width();

    // Save as PNG
    let output_path = PathBuf::from(output_dir).join(format!("{}.png", stem));
    let out_file = fs::File::create(output_path)?;
    image.write_png(out_file)?;

    // Store hotspot data
    let hotspot_info = json!({
        "hotspot_x": hotspot_x,
        "hotspot_y": hotspot_y,
        "width": width,
        "height": height,
        "source_file": filename
    });
    
    hotspot_data.insert(format!("{}.png", stem), hotspot_info);

    Ok(HotspotInfo {
        filename,
        hotspot_x,
        hotspot_y,
        width,
        height,
    })
}
