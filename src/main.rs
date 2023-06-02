use clap::Parser;
use std::fs::File;
use std::collections::{HashSet, HashMap, VecDeque};
use stl::read_stl;
use std::fs::OpenOptions;
use ordered_float::OrderedFloat;
use stl::Triangle;

#[derive(Parser)]
struct Cli {
    path: std::path::PathBuf,

    #[clap(long="output-folder")]
    output_folder: Option<std::path::PathBuf>,
}

type Vertex = [OrderedFloat<f32>; 3];


fn get_vertices(t: &Triangle) -> [Vertex; 3] {
    return [
        [OrderedFloat(t.v1[0]), OrderedFloat(t.v1[1]), OrderedFloat(t.v1[2])],
        [OrderedFloat(t.v2[0]), OrderedFloat(t.v2[1]), OrderedFloat(t.v2[2])],
        [OrderedFloat(t.v3[0]), OrderedFloat(t.v3[1]), OrderedFloat(t.v3[2])],
    ]
}



fn triangles_connected(a: &Triangle, b: &Triangle) -> bool {
    for p1 in [a.v1, a.v2, a.v3] {
        for p2 in [b.v1, b.v2, b.v3] {
            if p1 == p2 {
                return true
            }
        }
    }
    return false
}

fn copy_triangle(t: &stl::Triangle) -> stl::Triangle {
    stl::Triangle{
        normal: t.normal,
        v1: t.v1,
        v2: t.v2,
        v3: t.v3,
        attr_byte_count: t.attr_byte_count,
    }
}

fn find_connected_sets(triangles: &[Triangle]) -> Vec<Vec<Triangle>> {
    let mut vertex_to_triangles: HashMap<Vertex, Vec<usize>> = HashMap::new();
    let mut visited: HashSet<usize> = HashSet::new();
    let mut connected_sets: Vec<Vec<Triangle>> = Vec::new();

    // Step 2: Build the graph
    for (i, triangle) in triangles.iter().enumerate() {
        for vertex in get_vertices(triangle) {
            vertex_to_triangles
                .entry(vertex)
                .or_insert_with(Vec::new)
                .push(i);
        }
    }

    // Step 4: Find connected components using DFS with a stack
    for (i, triangle) in triangles.iter().enumerate() {
        if !visited.contains(&i) {
            let mut connected_set: Vec<Triangle> = Vec::new();
            let mut stack: VecDeque<usize> = VecDeque::new();
            stack.push_back(i);

            while let Some(current) = stack.pop_back() {
                if visited.contains(&current) {
                    continue;
                }

                visited.insert(current);
                let current_triangle = &triangles[current];
                connected_set.push(copy_triangle(current_triangle));

                for vertex in get_vertices(current_triangle) {
                    if let Some(adjacent_triangles) = vertex_to_triangles.get(&vertex) {
                        for &adjacent in adjacent_triangles {
                            if !visited.contains(&adjacent) {
                                stack.push_back(adjacent);
                            }
                        }
                    }
                }
            }

            connected_sets.push(connected_set);
        }
    }

    connected_sets
}

fn main() -> Result<(), std::io::Error> {
    let args = Cli::parse();

    println!("Loading {}...", args.path.display());

    let mut input_file = OpenOptions::new().read(true).open(args.path.clone())?;

    let stl = read_stl(&mut input_file)?;

    let mut graph: Vec<Vec<usize>> = vec![Vec::new(); stl.triangles.len()];
    let mut visited: Vec<bool> = vec![false; stl.triangles.len()];
    let connected_sets = find_connected_sets(&stl.triangles);



    let base_filename = args.path.file_stem().unwrap();
    // Use --output-folder flag if provided or fallback to the parent of the input
    let parent_path = args.output_folder.unwrap_or(args.path.parent().unwrap().to_path_buf());

    std::fs::create_dir_all(parent_path.clone())?;

    println!("Found {} separate solids in stl file. Creating new files...", connected_sets.len());

    for (i, triangle_list) in connected_sets.into_iter().enumerate() {
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
