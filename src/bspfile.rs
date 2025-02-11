use crate::*;
use binrw::io::Cursor;
use binrw::BinReaderExt;
use std::borrow::Cow;

pub struct BspFile<'a> {
    data: &'a [u8],
    directories: Directories,
    header: Header,
    #[allow(dead_code)]
    map_revision: u32,
}

impl<'a> BspFile<'a> {
    pub fn new(data: &'a [u8]) -> BspResult<Self> {
        const EXPECTED_HEADER: Header = Header {
            v: b'V',
            b: b'B',
            s: b'S',
            p: b'P',
        };
        // TODO: Use this to decide on the version to parse it as
        const EXPECTED_VERSION: u32 = 0x14;

        let mut cursor = Cursor::new(data);
        let header: Header = cursor.read_le()?;
        let version: u32 = cursor.read_le()?;

        if header != EXPECTED_HEADER || version != EXPECTED_VERSION {
            return Err(BspError::UnexpectedHeader(header));
        }

        let directories = cursor.read_le()?;

        let map_revision = cursor.read_le()?;

        Ok(BspFile {
            data,
            directories,
            header,
            map_revision,
        })
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn lump_reader(&self, lump: LumpType) -> BspResult<LumpReader<Cursor<Cow<[u8]>>>> {
        let (version, data) = self.get_lump(lump)?;
        Ok(LumpReader::new(data, version, lump))
    }

    pub fn get_lump(&self, lump_t: LumpType) -> BspResult<(u32, Cow<[u8]>)> {
        let lump = &self.directories[lump_t];
        let raw_data = self
            .data
            .get(lump.offset as usize..lump.offset as usize + lump.length as usize)
            .ok_or(BspError::LumpOutOfBounds(*lump))?;

        Ok(match lump.ident {
            0 => (lump.version, Cow::Borrowed(raw_data)),
            _ => {
                let data = lzma_decompress_with_header(raw_data, lump.ident as usize)?;
                (lump.version, Cow::Owned(data))
            }
        })
    }
}

#[allow(dead_code)]
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum LumpType {
    Entities,
    Planes,
    TextureData,
    Vertices,
    Visibility,
    Nodes,
    TextureInfo,
    Faces,
    Lighting,
    Occlusion,
    Leaves,
    FaceIds,
    Edges,
    SurfaceEdges,
    Models,
    WorldLights,
    LeafFaces,
    LeafBrushes,
    Brushes,
    BrushSides,
    Areas,
    AreaPortals,
    // Lump Portals sometimes?
    Unused0,
    // Lump clusters sometimes?
    // Lump prop hulls sometimes?
    Unused1,
    // Lump portal verts sometimes?
    // Lump prop hull verts sometimes?
    Unused2,
    // Lump cluster portals sometimes?
    // Lump prop tris sometimes?
    Unused3,
    DisplacementInfo,
    OriginalFaces,
    PhysDisplacement,
    PhysCollide,
    VertNormals,
    VertNormalIndices,
    DisplacementLightMapAlphas,
    DisplacementVertices,
    DisplacementLightMapSamplePositions,
    GameLump,
    LeafWaterData,
    Primitives,
    PrimVertices,
    PrimIndices,
    PakFile,
    ClipPortalVertices,
    CubeMaps,
    TextureDataStringData,
    TextureDataStringTable,
    Overlays,
    LeafMinimumDistanceToWater,
    FaceMacroTextureInfo,
    DisplacementTris,
    PhysicsCollideSurface,
    WaterOverlays,
    LeafAmbientIndexHdr,
    LeafAmbientIndex,
    LightingHdr,
    WorldLightsHdr,
    LeafAmbientLightingHdr,
    LeafAmbientLighting,
    XZipPakFile,
    FacesHdr,
    MapFlags,
    OverlayFades,
    OverlaySystemLevels,
    PhysLevel,
    DisplacementMultiBlend,
}

static_assertions::const_assert_eq!(LumpType::DisplacementMultiBlend as usize, 63);
