use bevy::render::{
    mesh::{Indices, Mesh, PrimitiveTopology, VertexAttributeValues},
    render_asset::RenderAssetUsages,
};
use curvo::prelude::{FloatingPoint, SurfaceTessellation};
use nalgebra::{allocator::Allocator, DefaultAllocator, DimName, DimNameDiff, DimNameSub, U1};

/// Bevy mesh generator for NURBS surfaces by curvo
pub struct NurbsSurfaceMesh<T: FloatingPoint, D: DimName>
where
    D: DimNameSub<U1>,
    DefaultAllocator: Allocator<T, D>,
    DefaultAllocator: Allocator<T, DimNameDiff<D, U1>>,
{
    tessellation: SurfaceTessellation<T, D>,
}

impl<T: FloatingPoint, D: DimName> NurbsSurfaceMesh<T, D>
where
    D: DimNameSub<U1>,
    DefaultAllocator: Allocator<T, D>,
    DefaultAllocator: Allocator<T, DimNameDiff<D, U1>>,
{
    pub fn new(tessellation: SurfaceTessellation<T, D>) -> Self {
        Self { tessellation }
    }

    /// Builds a triangle list mesh from the NURBS surface
    /// * `asset_usage` - The asset usage for the mesh. If None, default usage is used
    pub fn build_surface_triangle_list(&self, asset_usage: Option<RenderAssetUsages>) -> Mesh {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            asset_usage.unwrap_or_default(),
        );

        let to_array = Self::to_array_helper();

        let vertices = self
            .tessellation
            .points()
            .iter()
            .map(|v| to_array(v.coords.as_slice()))
            .collect();
        let normals = self
            .tessellation
            .normals()
            .iter()
            .map(|n| to_array(n.as_slice()))
            .collect();
        let uvs = self
            .tessellation
            .uvs()
            .iter()
            .map(|uv| [uv[0].to_f32().unwrap(), uv[1].to_f32().unwrap()])
            .collect();
        let indices = self
            .tessellation
            .faces()
            .iter()
            .flat_map(|f| f.iter().map(|i| *i as u32))
            .collect();
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            VertexAttributeValues::Float32x3(vertices),
        );
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            VertexAttributeValues::Float32x3(normals),
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, VertexAttributeValues::Float32x2(uvs));
        mesh.insert_indices(Indices::U32(indices));

        mesh
    }

    /// Builds a line list mesh from the normals
    /// * `normal_length` - The length of the normal lines. If None, original length is used, otherwise the normal is scaled to this length
    /// * `asset_usage` - The asset usage for the mesh. If None, default usage is used
    pub fn build_normal_line_list(
        &self,
        normal_length: Option<T>,
        asset_usage: Option<RenderAssetUsages>,
    ) -> Mesh {
        let mut line_list = Mesh::new(
            bevy::render::mesh::PrimitiveTopology::LineList,
            asset_usage.unwrap_or_default(),
        );
        let normals = self.tessellation.normals();

        let to_array = Self::to_array_helper();

        let vertices = self
            .tessellation
            .points()
            .iter()
            .enumerate()
            .flat_map(|(i, p)| {
                let n = &normals[i];
                let n = match normal_length {
                    Some(l) => n.normalize() * l,
                    _ => n.clone(),
                };
                let p0 = to_array(p.coords.as_slice());
                let p1 = to_array((p + n).coords.as_slice());
                [p0, p1]
            })
            .collect();

        line_list.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            VertexAttributeValues::Float32x3(vertices),
        );

        line_list
    }

    fn to_array_helper() -> impl Fn(&[T]) -> [f32; 3] {
        match D::dim() {
            1 => |slice: &[T]| -> [f32; 3] { [slice[0].to_f32().unwrap(), 0., 0.] },
            2 => |slice: &[T]| -> [f32; 3] {
                [slice[0].to_f32().unwrap(), slice[1].to_f32().unwrap(), 0.]
            },
            _ => |slice: &[T]| -> [f32; 3] {
                [
                    slice[0].to_f32().unwrap(),
                    slice[1].to_f32().unwrap(),
                    slice[2].to_f32().unwrap(),
                ]
            },
        }
    }
}

impl<T: FloatingPoint, D: DimName> From<SurfaceTessellation<T, D>> for NurbsSurfaceMesh<T, D>
where
    D: DimNameSub<U1>,
    DefaultAllocator: Allocator<T, D>,
    DefaultAllocator: Allocator<T, DimNameDiff<D, U1>>,
{
    fn from(tessellation: SurfaceTessellation<T, D>) -> Self {
        Self::new(tessellation)
    }
}

impl<T: FloatingPoint, D: DimName> From<NurbsSurfaceMesh<T, D>> for Mesh
where
    D: DimNameSub<U1>,
    DefaultAllocator: Allocator<T, D>,
    DefaultAllocator: Allocator<T, DimNameDiff<D, U1>>,
{
    fn from(value: NurbsSurfaceMesh<T, D>) -> Self {
        value.build_surface_triangle_list(None)
    }
}

impl<'a, T: FloatingPoint, D: DimName> From<&'a NurbsSurfaceMesh<T, D>> for Mesh
where
    D: DimNameSub<U1>,
    DefaultAllocator: Allocator<T, D>,
    DefaultAllocator: Allocator<T, DimNameDiff<D, U1>>,
{
    fn from(value: &'a NurbsSurfaceMesh<T, D>) -> Self {
        value.build_surface_triangle_list(None)
    }
}
