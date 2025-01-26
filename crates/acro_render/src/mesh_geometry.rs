use core::str;
use std::{rc::Rc, sync::Arc};

use acro_assets::{Assets, Loadable, LoaderContext};
use acro_math::{Vec2, Vec3};
use bytemuck::{Pod, Zeroable};
use eyre::OptionExt;
use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Vertex {
    pub position: Vec3,
    pub tex_coords: Vec2,
    pub normal: Vec3,
}

cfg_if::cfg_if! {
    if #[cfg(feature = "double-precision")] {
        const VEC3_FORMAT: wgpu::VertexFormat = wgpu::VertexFormat::Float64x3;
        const VEC2_FORMAT: wgpu::VertexFormat = wgpu::VertexFormat::Float64x2;
    } else {
        const VEC3_FORMAT: wgpu::VertexFormat = wgpu::VertexFormat::Float32x3;
        const VEC2_FORMAT: wgpu::VertexFormat = wgpu::VertexFormat::Float32x2;
    }
}

impl Vertex {
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VEC3_FORMAT,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<Vec3>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: VEC2_FORMAT,
                },
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<Vec2>() + std::mem::size_of::<Vec3>())
                        as wgpu::BufferAddress,
                    shader_location: 2,
                    format: VEC3_FORMAT,
                },
            ],
        }
    }
}

unsafe impl Zeroable for Vertex {}
unsafe impl Pod for Vertex {}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum MeshGeometryData {
    Embedded {
        vertices: Arc<[Vertex]>,
        indices: Arc<[u32]>,
    },
    ObjAsset(String),
}

impl MeshGeometryData {
    pub fn vertices(&self, assets: &Assets) -> Arc<[Vertex]> {
        match self {
            Self::Embedded { vertices, .. } => vertices.clone(),
            Self::ObjAsset(path) => assets.get::<ObjFile>(path).vertices.clone(),
        }
    }

    pub fn indices(&self, assets: &Assets) -> Arc<[u32]> {
        match self {
            Self::Embedded { indices, .. } => indices.clone(),
            Self::ObjAsset(path) => assets.get::<ObjFile>(path).indices.clone(),
        }
    }
}

#[derive(Debug)]
pub struct ObjFile {
    vertices: Arc<[Vertex]>,
    indices: Arc<[u32]>,
}

impl Loadable for ObjFile {
    type Config = ();

    fn load(ctx: &LoaderContext, config: Arc<Self::Config>, data: Vec<u8>) -> eyre::Result<Self> {
        let data = std::str::from_utf8(&data)?;

        let mut vertices = vec![];
        let mut indices = vec![];

        for line in data.lines() {
            let mut parts = line.split_whitespace();

            match parts.next() {
                Some("v") => {
                    let x: f32 = parts
                        .next()
                        .ok_or_eyre("invalid x position in vertex")?
                        .parse()?;
                    let y: f32 = parts
                        .next()
                        .ok_or_eyre("invalid y position in vertex")?
                        .parse()?;
                    let z: f32 = parts
                        .next()
                        .ok_or_eyre("invalid z position in vertex")?
                        .parse()?;
                    vertices.push(Vertex {
                        position: Vec3::new(x, y, z),
                        tex_coords: Vec2::zeros(),
                        normal: Vec3::zeros(),
                    });
                }
                Some("f") => {
                    let a: u32 = parts
                        .next()
                        .ok_or_eyre("invalid a vertex in face")?
                        .parse()?;
                    let b: u32 = parts
                        .next()
                        .ok_or_eyre("invalid b vertex in face")?
                        .parse()?;
                    let c: u32 = parts
                        .next()
                        .ok_or_eyre("invalid c vertex in face")?
                        .parse()?;

                    indices.push(a - 1);
                    indices.push(b - 1);
                    indices.push(c - 1);
                }
                _ => {}
            }
        }

        for face in indices.chunks_exact(3) {
            let a = vertices[face[0] as usize].position;
            let b = vertices[face[1] as usize].position;
            let c = vertices[face[2] as usize].position;

            let normal = (b - a).cross(&(c - a));

            for &i in face {
                vertices[i as usize].normal += normal;
            }
        }

        for vertex in &mut vertices {
            vertex.normal = vertex.normal.normalize();
        }

        // TODO: calculate normals

        Ok(Self {
            vertices: vertices.into(),
            indices: indices.into(),
        })
    }
}
