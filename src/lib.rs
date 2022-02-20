mod bspfile;
pub mod data;
mod error;
mod handle;
mod reader;

use crate::bspfile::LumpType;
pub use crate::data::TextureFlags;
pub use crate::data::Vector;
use crate::data::*;
use crate::handle::Handle;
use binrw::io::Cursor;
use binrw::BinRead;
use bspfile::BspFile;
pub use error::{BspError, StringError};
use reader::LumpReader;
use std::{io::Read, ops::Deref};

pub type BspResult<T> = Result<T, BspError>;

#[derive(Debug, Clone)]
pub struct Leaves {
    leaves: Vec<Leaf>,
}

impl Leaves {
    pub fn new(mut leaves: Vec<Leaf>) -> Self {
        leaves.sort_unstable_by_key(|leaf| leaf.cluster);

        Leaves { leaves }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Leaf> {
        self.into_iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Leaf> {
        self.into_iter()
    }

    pub fn into_inner(self) -> Vec<Leaf> {
        self.leaves
    }

    pub fn clusters(&self) -> impl Iterator<Item = impl Iterator<Item = &Leaf>> {
        LeafClusters {
            leaves: &self.leaves,
            index: 0,
        }
    }
}

struct LeafClusters<'a> {
    leaves: &'a [Leaf],
    index: usize,
}

impl<'a> Iterator for LeafClusters<'a> {
    type Item = <&'a [Leaf] as IntoIterator>::IntoIter;

    fn next(&mut self) -> Option<Self::Item> {
        let cluster = self.leaves.get(self.index)?.cluster;
        let remaining_leaves = self.leaves.get(self.index..)?;
        let cluster_size = remaining_leaves
            .iter()
            .take_while(|leaf| leaf.cluster == cluster)
            .count();
        self.index += cluster_size;
        Some(remaining_leaves[0..cluster_size].iter())
    }
}

#[test]
fn test_leaf_clusters() {
    let leaves: Leaves = vec![
        Leaf {
            contents: 0,
            cluster: 0,
            ..Default::default()
        },
        Leaf {
            contents: 1,
            cluster: 0,
            ..Default::default()
        },
        Leaf {
            contents: 2,
            cluster: 1,
            ..Default::default()
        },
        Leaf {
            contents: 3,
            cluster: 2,
            ..Default::default()
        },
        Leaf {
            contents: 4,
            cluster: 2,
            ..Default::default()
        },
    ]
    .into();

    let clustered: Vec<Vec<i32>> = leaves
        .clusters()
        .map(|cluster| cluster.map(|leaf| leaf.contents).collect())
        .collect();
    assert_eq!(vec![vec![0, 1], vec![2], vec![3, 4],], clustered);
}

impl From<Vec<Leaf>> for Leaves {
    fn from(other: Vec<Leaf>) -> Self {
        Self::new(other)
    }
}

impl Deref for Leaves {
    type Target = [Leaf];

    fn deref(&self) -> &Self::Target {
        &self.leaves
    }
}

impl IntoIterator for Leaves {
    type Item = Leaf;
    type IntoIter = <Vec<Leaf> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.leaves.into_iter()
    }
}

impl<'a> IntoIterator for &'a Leaves {
    type Item = &'a Leaf;
    type IntoIter = <&'a [Leaf] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        (&self.leaves[..]).iter()
    }
}

impl<'a> IntoIterator for &'a mut Leaves {
    type Item = &'a mut Leaf;
    type IntoIter = <&'a mut [Leaf] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.leaves.iter_mut()
    }
}

// TODO: Store all the allocated objects inline to improve cache usage
/// A parsed bsp file
#[derive(Debug)]
pub struct Bsp {
    pub header: Header,
    pub entities: Entities,
    pub textures_data: Vec<TextureData>,
    pub textures_info: Vec<TextureInfo>,
    pub planes: Vec<Plane>,
    pub nodes: Vec<Node>,
    pub leaves: Leaves,
    pub leaf_faces: Vec<LeafFace>,
    pub leaf_brushes: Vec<LeafBrush>,
    pub models: Vec<Model>,
    pub brushes: Vec<Brush>,
    pub brush_sides: Vec<BrushSide>,
    pub vertices: Vec<Vertex>,
    pub edges: Vec<Edge>,
    pub surface_edges: Vec<SurfaceEdge>,
    pub faces: Vec<Face>,
    pub original_faces: Vec<Face>,
    pub vis_data: VisData,
    pub displacements: Vec<DisplacementInfo>,
}

impl Bsp {
    pub fn read(data: &[u8]) -> BspResult<Self> {
        let bsp_file = BspFile::new(data)?;

        let entities = bsp_file.lump_reader(LumpType::Entities)?.read_entities()?;
        let textures_data = bsp_file
            .lump_reader(LumpType::TextureData)?
            .read_vec(|r| r.read())?;
        let textures_info = bsp_file
            .lump_reader(LumpType::TextureInfo)?
            .read_vec(|r| r.read())?;
        let planes = bsp_file
            .lump_reader(LumpType::Planes)?
            .read_vec(|r| r.read())?;
        let nodes = bsp_file
            .lump_reader(LumpType::Nodes)?
            .read_vec(|r| r.read())?;
        let leaves = bsp_file
            .lump_reader(LumpType::Leaves)?
            .read_vec(|r| r.read())?
            .into();
        let leaf_faces = bsp_file
            .lump_reader(LumpType::LeafFaces)?
            .read_vec(|r| r.read())?;
        let leaf_brushes = bsp_file
            .lump_reader(LumpType::LeafBrushes)?
            .read_vec(|r| r.read())?;
        let models = bsp_file
            .lump_reader(LumpType::Models)?
            .read_vec(|r| r.read())?;
        let brushes = bsp_file
            .lump_reader(LumpType::Brushes)?
            .read_vec(|r| r.read())?;
        let brush_sides = bsp_file
            .lump_reader(LumpType::BrushSides)?
            .read_vec(|r| r.read())?;
        let vertices = bsp_file
            .lump_reader(LumpType::Vertices)?
            .read_vec(|r| r.read())?;
        let edges = bsp_file
            .lump_reader(LumpType::Edges)?
            .read_vec(|r| r.read())?;
        let surface_edges = bsp_file
            .lump_reader(LumpType::SurfaceEdges)?
            .read_vec(|r| r.read())?;
        let faces = bsp_file
            .lump_reader(LumpType::Faces)?
            .read_vec(|r| r.read())?;
        let original_faces = bsp_file
            .lump_reader(LumpType::OriginalFaces)?
            .read_vec(|r| r.read())?;
        let vis_data = bsp_file.lump_reader(LumpType::Visibility)?.read_visdata()?;
        let displacements = bsp_file
            .lump_reader(LumpType::DisplacementInfo)?
            .read_vec(|r| r.read())?;

        Ok({
            Bsp {
                header: bsp_file.header().clone(),
                entities,
                textures_data,
                textures_info,
                planes,
                nodes,
                leaves,
                leaf_faces,
                leaf_brushes,
                models,
                brushes,
                brush_sides,
                vertices,
                edges,
                surface_edges,
                faces,
                original_faces,
                vis_data,
                displacements,
            }
        })
    }

    pub fn leaf(&self, n: usize) -> Option<Handle<'_, Leaf>> {
        self.leaves.get(n).map(|leaf| Handle::new(self, leaf))
    }

    pub fn plane(&self, n: usize) -> Option<Handle<'_, Plane>> {
        self.planes.get(n).map(|plane| Handle::new(self, plane))
    }

    pub fn face(&self, n: usize) -> Option<Handle<'_, Face>> {
        self.faces.get(n).map(|face| Handle::new(self, face))
    }

    pub fn node(&self, n: usize) -> Option<Handle<'_, Node>> {
        self.nodes.get(n).map(|node| Handle::new(self, node))
    }

    pub fn displacement(&self, n: usize) -> Option<Handle<'_, DisplacementInfo>> {
        self.displacements
            .get(n)
            .map(|displacement| Handle::new(self, displacement))
    }

    /// Get the root node of the bsp
    pub fn root_node(&self) -> Option<Handle<'_, Node>> {
        self.node(0)
    }

    /// Get all models stored in the bsp
    pub fn models(&self) -> impl Iterator<Item = Handle<'_, Model>> {
        self.models.iter().map(move |m| Handle::new(self, m))
    }

    /// Find a leaf for a specific position
    pub fn leaf_at(&self, point: Vector) -> Option<Handle<'_, Leaf>> {
        let mut current = self.root_node()?;

        loop {
            let plane = current.plane()?;
            let dot: f32 = point
                .iter()
                .zip(plane.normal.iter())
                .map(|(a, b)| a * b)
                .sum();

            let [front, back] = current.children;

            let next = if dot < plane.dist { back } else { front };

            if next < 0 {
                return self.leaf((!next) as usize);
            } else {
                current = self.node(next as usize)?;
            }
        }
    }

    /// Get all faces stored in the bsp
    pub fn original_faces(&self) -> impl Iterator<Item = Handle<Face>> {
        self.faces.iter().map(move |face| Handle::new(self, face))
    }
}

#[cfg(test)]
mod tests {
    use super::Bsp;

    #[test]
    fn tf2_file() {
        use std::fs::read;

        let data = read("koth_bagel_rc2a.bsp").unwrap();

        Bsp::read(&data).unwrap();
    }
}
