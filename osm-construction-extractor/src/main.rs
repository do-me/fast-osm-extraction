use anyhow::Result;
use clap::Parser;
use geo::{Coord, LineString};
use indicatif::{ProgressBar, ProgressStyle};
use osmpbfreader::{OsmId, OsmObj, OsmPbfReader, WayId};
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::path::PathBuf;
use std::time::Instant;

// A simplified struct to hold our final extracted data in memory
#[derive(Debug)]
struct ConstructionWay {
    id: WayId,
    tags: HashMap<String, String>,
    geometry: LineString,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the input OSM PBF file
    #[arg(short, long)]
    input: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let start_time = Instant::now();

    println!("-> Opening PBF file: {:?}", &args.input);
    let f = File::open(&args.input)?;
    let mut reader = OsmPbfReader::new(f);

    // Optimized predicate with early exits and string slice lookups
    let predicate = |obj: &OsmObj| -> bool {
        match obj.way() {
            Some(way) if way.nodes.len() >= 2 => {
                // Use string slices which work with SmartString's Borrow<str> implementation
                way.tags.contains_key("highway") && way.tags.contains_key("construction")
            }
            _ => false,
        }
    };

    println!("-> Pass 1: Finding ways and collecting dependencies...");
    let objects: BTreeMap<OsmId, OsmObj> = reader.get_objs_and_deps(predicate)?;
    let extraction_duration = start_time.elapsed();
    println!(
        "   Found {} total objects (ways and their required nodes) in {:.2?}.",
        objects.len(),
        extraction_duration
    );

    println!("-> Pass 2: Re-structuring extracted data into final format...");
    let processing_start_time = Instant::now();

    // Pre-filter and collect ways more efficiently
    let ways_to_process: Vec<&osmpbfreader::Way> = objects
        .values()
        .filter_map(|obj| {
            if let OsmObj::Way(way) = obj {
                // Check for construction highway ways
                if way.tags.contains_key("highway") && way.tags.contains_key("construction") {
                    Some(way)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    let bar = ProgressBar::new(ways_to_process.len() as u64);
    bar.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>7}/{len:7} ({eta})")?
        .progress_chars("#>-"));

    // Pre-allocate with exact capacity
    let mut final_ways: Vec<ConstructionWay> = Vec::with_capacity(ways_to_process.len());

    for way in ways_to_process {
        // Pre-allocate coordinate vector with exact capacity
        let mut coords: Vec<Coord> = Vec::with_capacity(way.nodes.len());
        let mut valid_way = true;
        
        // Process nodes in batch for better cache locality
        for &node_id in &way.nodes {
            match objects.get(&node_id.into()) {
                Some(OsmObj::Node(node)) => {
                    coords.push(Coord { x: node.lon(), y: node.lat() });
                }
                _ => {
                    eprintln!("Warning: Node ID {:?} for Way ID {:?} not found. Skipping.", node_id, way.id);
                    valid_way = false;
                    break;
                }
            }
        }

        if !valid_way {
            bar.inc(1);
            continue;
        }

        // More efficient tag conversion with pre-allocated capacity
        let mut tags_map: HashMap<String, String> = HashMap::with_capacity(way.tags.len());
        way.tags.iter().for_each(|(k, v)| {
            tags_map.insert(k.to_string(), v.to_string());
        });

        final_ways.push(ConstructionWay {
            id: way.id,
            tags: tags_map,
            geometry: LineString(coords),
        });

        bar.inc(1);
    }
    bar.finish_with_message("Done processing ways.");

    let processing_duration = processing_start_time.elapsed();
    let total_duration = start_time.elapsed();

    println!("\n--- BENCHMARK RESULTS ---");
    println!("Total ways extracted: {}", final_ways.len());
    println!("Core extraction (PBF read & dependency resolution): {:.2?}", extraction_duration);
    println!("Data restructuring (geometry building, etc.):       {:.2?}", processing_duration);
    println!("----------------------------------------------------");
    println!("Total runtime:                                      {:.2?}", total_duration);
    println!("\n✅ Success! Data is held in an in-memory array.");

    // We can even print one to prove it exists
    if let Some(first_way) = final_ways.first() {
        println!("\nExample of first extracted way:");
        println!("{:#?}", first_way);
    }

    Ok(())
}