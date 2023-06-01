use clap::Parser;
use std::fs::File;
use std::collections::HashSet;
use stl::read_stl;
use std::fs::OpenOptions;
use ordered_float::OrderedFloat;

#[derive(Parser)]
struct Cli {
    path: std::path::PathBuf,

    #[clap(long="output-folder")]
    output_folder: Option<std::path::PathBuf>,
}

type Point = [OrderedFloat<f32>; 3];


// stl::Triangle does not implement Copy or Clone even though it probably could
// This seemed to be the simplest way to get a quick copy of this object
// There may be a better way
fn copy_triangle(t: &stl::Triangle) -> stl::Triangle {
    stl::Triangle{
        normal: t.normal,
        v1: t.v1,
        v2: t.v2,
        v3: t.v3,
        attr_byte_count: t.attr_byte_count,
    }
}

fn main() -> Result<(), std::io::Error> {
    let args = Cli::parse();

    println!("Loading {}...", args.path.display());

    let mut input_file = OpenOptions::new().read(true).open(args.path.clone())?;

    let stl = read_stl(&mut input_file)?;

    let mut solids: Vec<HashSet<Point>> = vec![];
    let mut triangles: Vec<Vec<stl::Triangle>> = vec![];

    for t in stl.triangles.iter() {
        // Use OrderedFloat because they can be used in a HashSet
        let p1 = [OrderedFloat(t.v1[0]), OrderedFloat(t.v1[1]), OrderedFloat(t.v1[2])];
        let p2 = [OrderedFloat(t.v2[0]), OrderedFloat(t.v2[1]), OrderedFloat(t.v2[2])];
        let p3 = [OrderedFloat(t.v3[0]), OrderedFloat(t.v3[1]), OrderedFloat(t.v3[2])];

        println!("{:?} {:?} {:?}", p1, p2, p3);

        let mut found = false;
        for (i, solid) in solids.iter_mut().enumerate() {
            if solid.get(&p1).is_some() || solid.get(&p2).is_some() || solid.get(&p3).is_some() {
                // At least one point in the triangle is already attached to this solid.
                // Add all the points to the list of points.
                solid.insert(p1);
                solid.insert(p2);
                solid.insert(p3);

                triangles[i].push(copy_triangle(t));

                found = true;
                break;
            }
        }
        if !found {
            println!("Triangle not found in existing solids. Creating new solid");
            let mut new_solid = HashSet::with_capacity(3);
            new_solid.insert(p1);
            new_solid.insert(p2);
            new_solid.insert(p3);
            solids.push(new_solid);

            triangles.push(vec![copy_triangle(t)]);
        }
    }

    let base_filename = args.path.file_stem().unwrap();
    // Use --output-folder flag if provided or fallback to the parent of the input
    let parent_path = args.output_folder.unwrap_or(args.path.parent().unwrap().to_path_buf());

    std::fs::create_dir_all(parent_path.clone())?;

    println!("Found {} separate solids in stl file. Creating new files...", solids.len());

    for (i, triangle_list) in triangles.into_iter().enumerate() {
        let new_stl = stl::BinaryStlFile{
            header: stl::BinaryStlHeader{
                header: stl.header.header.clone(),
                num_triangles: triangle_list.len() as u32,
            },
            triangles: triangle_list,
        };

        let mut output_path = parent_path.clone();
        output_path.push(format!("{}_{:04}.stl", base_filename.to_str().unwrap(), i));
        println!("Writing new stl file to {}", output_path.display());

        let mut file = File::create(output_path)?;

        stl::write_stl(&mut file, &new_stl)?;
    }

    Ok(())
}
