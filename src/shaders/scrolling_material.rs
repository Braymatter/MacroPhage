use crate::{MaterialPipeline, SpecializedMaterial};
use bevy::asset::{AssetServer, Handle};
use bevy::ecs::system::{lifetimeless::SRes, SystemParamItem};
use bevy::math::Vec4;
use bevy::pbr::{AlphaMode};
use bevy::prelude::{App, Commands, MaterialPlugin, Plugin, Res, ResMut, Time};
use bevy::reflect::TypeUuid;
use bevy::render::{color::Color, mesh::MeshVertexBufferLayout, prelude::Shader, render_asset::{PrepareAssetError, RenderAsset, RenderAssets}, render_resource::{
    std140::{AsStd140, Std140},
    *,
}, RenderApp, renderer::RenderDevice, RenderStage, texture::Image};
use bevy::render::renderer::RenderQueue;

// TODO: aspect-aware changes
// TODO: BB-aware changes
// TODO: correct blending mode
// TODO: more uniforms for scrolling settings other than "on/off"

/// A material extending normal PBR lighting which allows an overlaid texture to "scroll"
/// over the object using screen-space coordinates.
///
/// Use:
///    Import the plugin, then configure a new material mesh bundle to use, e.g.:
///       commands.spawn().insert_bundle(MaterialMeshBundle {
//             mesh: meshes.add(Mesh::from(shape::UVSphere { radius: 2.0, ..default() })),
//             material: materials.add(ScrollingPbrMaterial {
//                 base_color: Color::rgb(1.0, 1.0, 0.8),
//                 base_color_texture: Some(texture_handle),
//                 scrolling_texture: Some(overlay_handle),
//                 alpha_mode: AlphaMode::Blend,
//                 ..default()
//             }),
//             transform: Transform::from_xyz(0.0, 0.5, 0.0),
//             ..default()
//         });
#[derive(Debug, Clone, TypeUuid)]
#[uuid = "7494888b-c082-457b-aacf-517228cc0c23"]
pub struct ScrollingPbrMaterial {
    /// Doubles as diffuse albedo for non-metallic, specular for metallic and a mix for everything
    /// in between. If used together with a base_color_texture, this is factored into the final
    /// base color as `base_color * base_color_texture_value`
    pub base_color: Color,
    pub base_color_texture: Option<Handle<Image>>,
    // Use a color for user friendliness even though we technically don't use the alpha channel
    // Might be used in the future for exposure correction in HDR
    pub emissive: Color,
    pub emissive_texture: Option<Handle<Image>>,
    /// Linear perceptual roughness, clamped to [0.089, 1.0] in the shader
    /// Defaults to minimum of 0.089
    /// If used together with a roughness/metallic texture, this is factored into the final base
    /// color as `roughness * roughness_texture_value`
    pub perceptual_roughness: f32,
    /// From [0.0, 1.0], dielectric to pure metallic
    /// If used together with a roughness/metallic texture, this is factored into the final base
    /// color as `metallic * metallic_texture_value`
    pub metallic: f32,
    pub metallic_roughness_texture: Option<Handle<Image>>,
    /// Specular intensity for non-metals on a linear scale of [0.0, 1.0]
    /// defaults to 0.5 which is mapped to 4% reflectance in the shader
    pub reflectance: f32,
    pub normal_map_texture: Option<Handle<Image>>,
    /// Normal map textures authored for DirectX have their y-component flipped. Set this to flip
    /// it to right-handed conventions.
    pub flip_normal_map_y: bool,
    pub occlusion_texture: Option<Handle<Image>>,
    /// Support two-sided lighting by automatically flipping the normals for "back" faces
    /// within the PBR lighting shader.
    /// Defaults to false.
    /// This does not automatically configure backface culling, which can be done via
    /// `cull_mode`.
    pub double_sided: bool,
    /// Whether to cull the "front", "back" or neither side of a mesh
    /// defaults to `Face::Back`
    pub cull_mode: Option<Face>,
    pub unlit: bool,
    pub alpha_mode: AlphaMode,
    /// Time data for animation purposes
    pub time: f32,
    /// Texture for the scrolling overlay
    pub scrolling_enabled: bool,
    pub scrolling_texture: Option<Handle<Image>>,
}

impl Default for ScrollingPbrMaterial {
    fn default() -> Self {
        ScrollingPbrMaterial {
            base_color: Color::rgb(1.0, 1.0, 1.0),
            base_color_texture: None,
            emissive: Color::BLACK,
            emissive_texture: None,
            // This is the minimum the roughness is clamped to in shader code
            // See <https://google.github.io/filament/Filament.html#materialsystem/parameterization/>
            // It's the minimum floating point value that won't be rounded down to 0 in the
            // calculations used. Although technically for 32-bit floats, 0.045 could be
            // used.
            perceptual_roughness: 0.089,
            // Few materials are purely dielectric or metallic
            // This is just a default for mostly-dielectric
            metallic: 0.01,
            metallic_roughness_texture: None,
            // Minimum real-world reflectance is 2%, most materials between 2-5%
            // Expressed in a linear scale and equivalent to 4% reflectance see
            // <https://google.github.io/filament/Material%20Properties.pdf>
            reflectance: 0.5,
            occlusion_texture: None,
            normal_map_texture: None,
            flip_normal_map_y: false,
            double_sided: false,
            cull_mode: Some(Face::Back),
            unlit: false,
            alpha_mode: AlphaMode::Opaque,
            time: 0.,
            scrolling_enabled: true,
            scrolling_texture: None
        }
    }
}

impl From<Color> for ScrollingPbrMaterial {
    fn from(color: Color) -> Self {
        ScrollingPbrMaterial {
            base_color: color,
            ..Default::default()
        }
    }
}

impl From<Handle<Image>> for ScrollingPbrMaterial {
    fn from(texture: Handle<Image>) -> Self {
        ScrollingPbrMaterial {
            base_color_texture: Some(texture),
            ..Default::default()
        }
    }
}

// NOTE: These must match the bit flags in bevy_pbr/src/render/scrolling_material.wgsl!
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct StandardMaterialFlags: u32 {
        const BASE_COLOR_TEXTURE         = (1 << 0);
        const EMISSIVE_TEXTURE           = (1 << 1);
        const METALLIC_ROUGHNESS_TEXTURE = (1 << 2);
        const OCCLUSION_TEXTURE          = (1 << 3);
        const DOUBLE_SIDED               = (1 << 4);
        const UNLIT                      = (1 << 5);
        const ALPHA_MODE_OPAQUE          = (1 << 6);
        const ALPHA_MODE_MASK            = (1 << 7);
        const ALPHA_MODE_BLEND           = (1 << 8);
        const TWO_COMPONENT_NORMAL_MAP   = (1 << 9);
        const FLIP_NORMAL_MAP_Y          = (1 << 10);
        const SCROLLING_ENABLED          = (1 << 11);
        const NONE                       = 0;
        const UNINITIALIZED              = 0xFFFF;
    }
}

/// The GPU representation of the uniform data of a [`StandardMaterial`].
#[derive(Debug, Clone, Default, AsStd140)]
pub struct StandardMaterialUniformData {
    /// Doubles as diffuse albedo for non-metallic, specular for metallic and a mix for everything
    /// in between.
    pub base_color: Vec4,
    // Use a color for user friendliness even though we technically don't use the alpha channel
    // Might be used in the future for exposure correction in HDR
    pub emissive: Vec4,
    /// Linear perceptual roughness, clamped to [0.089, 1.0] in the shader
    /// Defaults to minimum of 0.089
    pub roughness: f32,
    /// From [0.0, 1.0], dielectric to pure metallic
    pub metallic: f32,
    /// Specular intensity for non-metals on a linear scale of [0.0, 1.0]
    /// defaults to 0.5 which is mapped to 4% reflectance in the shader
    pub reflectance: f32,
    pub flags: u32,
    /// When the alpha mode mask flag is set, any base color alpha above this cutoff means fully opaque,
    /// and any below means fully transparent.
    pub alpha_cutoff: f32,
    /// Time data for animation purposes
    pub time: f32,
}

/// The GPU representation of a [`StandardMaterial`].
#[derive(Debug, Clone)]
pub struct GpuStandardMaterial {
    /// A buffer containing the [`StandardMaterialUniformData`] of the material.
    pub buffer: Buffer,
    /// Data to eventually put into the buffer
    pub uniform: StandardMaterialUniformData,
    /// The bind group specifying how the [`StandardMaterialUniformData`] and
    /// all the textures of the material are bound.
    pub bind_group: BindGroup,
    pub has_normal_map: bool,
    pub flags: StandardMaterialFlags,
    pub base_color_texture: Option<Handle<Image>>,
    pub alpha_mode: AlphaMode,
    pub cull_mode: Option<Face>,
}

impl RenderAsset for ScrollingPbrMaterial {
    type ExtractedAsset = ScrollingPbrMaterial;
    type PreparedAsset = GpuStandardMaterial;
    type Param = (
        SRes<RenderDevice>,
        SRes<MaterialPipeline<ScrollingPbrMaterial>>,
        SRes<RenderAssets<Image>>,
    );

    fn extract_asset(&self) -> Self::ExtractedAsset {
        self.clone()
    }

    // prep binding
    fn prepare_asset(
        material: Self::ExtractedAsset,
        (render_device, pbr_pipeline, gpu_images): &mut SystemParamItem<Self::Param>,
    ) -> Result<Self::PreparedAsset, PrepareAssetError<Self::ExtractedAsset>> {
        let (base_color_texture_view, base_color_sampler) = if let Some(result) = pbr_pipeline
            .mesh_pipeline
            .get_image_texture(gpu_images, &material.base_color_texture)
        {
            result
        } else {
            return Err(PrepareAssetError::RetryNextUpdate(material));
        };

        let (emissive_texture_view, emissive_sampler) = if let Some(result) = pbr_pipeline
            .mesh_pipeline
            .get_image_texture(gpu_images, &material.emissive_texture)
        {
            result
        } else {
            return Err(PrepareAssetError::RetryNextUpdate(material));
        };

        let (metallic_roughness_texture_view, metallic_roughness_sampler) = if let Some(result) =
            pbr_pipeline
                .mesh_pipeline
                .get_image_texture(gpu_images, &material.metallic_roughness_texture)
        {
            result
        } else {
            return Err(PrepareAssetError::RetryNextUpdate(material));
        };
        let (normal_map_texture_view, normal_map_sampler) = if let Some(result) = pbr_pipeline
            .mesh_pipeline
            .get_image_texture(gpu_images, &material.normal_map_texture)
        {
            result
        } else {
            return Err(PrepareAssetError::RetryNextUpdate(material));
        };
        let (occlusion_texture_view, occlusion_sampler) = if let Some(result) = pbr_pipeline
            .mesh_pipeline
            .get_image_texture(gpu_images, &material.occlusion_texture)
        {
            result
        } else {
            return Err(PrepareAssetError::RetryNextUpdate(material));
        };
        let (scrolling_texture_view, scrolling_texture_sampler) = if let Some(result) = pbr_pipeline
            .mesh_pipeline
            .get_image_texture(gpu_images, &material.scrolling_texture)
        {
            result
        } else {
            return Err(PrepareAssetError::RetryNextUpdate(material));
        };

        let mut flags = StandardMaterialFlags::NONE;
        if material.base_color_texture.is_some() {
            flags |= StandardMaterialFlags::BASE_COLOR_TEXTURE;
        }
        if material.emissive_texture.is_some() {
            flags |= StandardMaterialFlags::EMISSIVE_TEXTURE;
        }
        if material.metallic_roughness_texture.is_some() {
            flags |= StandardMaterialFlags::METALLIC_ROUGHNESS_TEXTURE;
        }
        if material.occlusion_texture.is_some() {
            flags |= StandardMaterialFlags::OCCLUSION_TEXTURE;
        }
        if material.double_sided {
            flags |= StandardMaterialFlags::DOUBLE_SIDED;
        }
        if material.unlit {
            flags |= StandardMaterialFlags::UNLIT;
        }
        let has_normal_map = material.normal_map_texture.is_some();
        if has_normal_map {
            match gpu_images
                .get(material.normal_map_texture.as_ref().unwrap())
                .unwrap()
                .texture_format
            {
                // All 2-component unorm formats
                TextureFormat::Rg8Unorm
                | TextureFormat::Rg16Unorm
                | TextureFormat::Bc5RgUnorm
                | TextureFormat::EacRg11Unorm => {
                    flags |= StandardMaterialFlags::TWO_COMPONENT_NORMAL_MAP
                }
                _ => {}
            }
            if material.flip_normal_map_y {
                flags |= StandardMaterialFlags::FLIP_NORMAL_MAP_Y;
            }
        }
        if material.scrolling_enabled {
            flags |= StandardMaterialFlags::SCROLLING_ENABLED;
        }
        // NOTE: 0.5 is from the glTF default - do we want this?
        let mut alpha_cutoff = 0.5;
        match material.alpha_mode {
            AlphaMode::Opaque => flags |= StandardMaterialFlags::ALPHA_MODE_OPAQUE,
            AlphaMode::Mask(c) => {
                alpha_cutoff = c;
                flags |= StandardMaterialFlags::ALPHA_MODE_MASK;
            }
            AlphaMode::Blend => flags |= StandardMaterialFlags::ALPHA_MODE_BLEND,
        };

        let value = StandardMaterialUniformData {
            base_color: material.base_color.as_linear_rgba_f32().into(),
            emissive: material.emissive.into(),
            roughness: material.perceptual_roughness,
            metallic: material.metallic,
            reflectance: material.reflectance,
            flags: flags.bits(),
            alpha_cutoff,
            time: material.time,
        };
        let value_std140 = value.as_std140();

        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("pbr_standard_material_uniform_buffer"),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            contents: value_std140.as_bytes(),
        });
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(base_color_texture_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(base_color_sampler),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::TextureView(emissive_texture_view),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: BindingResource::Sampler(emissive_sampler),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: BindingResource::TextureView(metallic_roughness_texture_view),
                },
                BindGroupEntry {
                    binding: 6,
                    resource: BindingResource::Sampler(metallic_roughness_sampler),
                },
                BindGroupEntry {
                    binding: 7,
                    resource: BindingResource::TextureView(occlusion_texture_view),
                },
                BindGroupEntry {
                    binding: 8,
                    resource: BindingResource::Sampler(occlusion_sampler),
                },
                BindGroupEntry {
                    binding: 9,
                    resource: BindingResource::TextureView(normal_map_texture_view),
                },
                BindGroupEntry {
                    binding: 10,
                    resource: BindingResource::Sampler(normal_map_sampler),
                },
                BindGroupEntry {
                    binding: 11,
                    resource: BindingResource::TextureView(scrolling_texture_view),
                },
                BindGroupEntry {
                    binding: 12,
                    resource: BindingResource::Sampler(scrolling_texture_sampler),
                },
            ],
            label: Some("pbr_standard_material_bind_group"),
            layout: &pbr_pipeline.material_layout,
        });

        Ok(GpuStandardMaterial {
            buffer,
            uniform: value,
            bind_group,
            flags,
            has_normal_map,
            base_color_texture: material.base_color_texture,
            alpha_mode: material.alpha_mode,
            cull_mode: material.cull_mode,
        })
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct StandardMaterialKey {
    normal_map: bool,
    cull_mode: Option<Face>,
}

impl SpecializedMaterial for ScrollingPbrMaterial {
    type Key = StandardMaterialKey;

    fn key(render_asset: &<Self as RenderAsset>::PreparedAsset) -> Self::Key {
        StandardMaterialKey {
            normal_map: render_asset.has_normal_map,
            cull_mode: render_asset.cull_mode,
        }
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        key: Self::Key,
        _layout: &MeshVertexBufferLayout,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if key.normal_map {
            descriptor
                .fragment
                .as_mut()
                .unwrap()
                .shader_defs
                .push(String::from("STANDARDMATERIAL_NORMAL_MAP"));
        }
        descriptor.primitive.cull_mode = key.cull_mode;
        if let Some(label) = &mut descriptor.label {
            *label = format!("pbr_{}", *label).into();
        }
        Ok(())
    }

    fn vertex_shader(_asset_server: &AssetServer) -> Option<Handle<Shader>> {
        Some(_asset_server.load("shaders/scrolling_material.wgsl"))
    }

    fn fragment_shader(_asset_server: &AssetServer) -> Option<Handle<Shader>> {
        Some(_asset_server.load("shaders/scrolling_material.wgsl"))
    }

    #[inline]
    fn bind_group(render_asset: &<Self as RenderAsset>::PreparedAsset) -> &BindGroup {
        &render_asset.bind_group
    }

    fn bind_group_layout(
        render_device: &RenderDevice,
    ) -> BindGroupLayout {
        render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new(
                            StandardMaterialUniformData::std140_size_static() as u64,
                        ),
                    },
                    count: None,
                },
                // Base Color Texture
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                // Base Color Texture Sampler
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                // Emissive Texture
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                // Emissive Texture Sampler
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                // Metallic Roughness Texture
                BindGroupLayoutEntry {
                    binding: 5,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                // Metallic Roughness Texture Sampler
                BindGroupLayoutEntry {
                    binding: 6,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                // Occlusion Texture
                BindGroupLayoutEntry {
                    binding: 7,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                // Occlusion Texture Sampler
                BindGroupLayoutEntry {
                    binding: 8,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                // Normal Map Texture
                BindGroupLayoutEntry {
                    binding: 9,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                // Normal Map Texture Sampler
                BindGroupLayoutEntry {
                    binding: 10,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                // Scrolling Overlay Texture
                BindGroupLayoutEntry {
                    binding: 11,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                // Scrolling Overlay Texture Sampler
                BindGroupLayoutEntry {
                    binding: 12,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("pbr_material_layout"),
        })
    }

    #[inline]
    fn alpha_mode(render_asset: &<Self as RenderAsset>::PreparedAsset) -> AlphaMode {
        render_asset.alpha_mode
    }
}

pub struct ScrollingPbrMaterialPlugin;

impl Plugin for ScrollingPbrMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<ScrollingPbrMaterial>::default());

        app.sub_app_mut(RenderApp)
            .add_system_to_stage(RenderStage::Extract, extract_time)
            .add_system_to_stage(RenderStage::Prepare, prepare_scrolling_material);
    }
}

#[derive(Default)]
struct ExtractedTime {
    elapsed_time: f32,
}

fn extract_time(mut commands: Commands, time: Res<Time>) {
    commands.insert_resource(ExtractedTime {
        elapsed_time: time.seconds_since_startup() as f32,
    });
}

fn prepare_scrolling_material(
    mut material_assets: ResMut<RenderAssets<ScrollingPbrMaterial>>,
    time: Res<ExtractedTime>,
    render_queue: Res<RenderQueue>,
) {
    for material in material_assets.values_mut() {
        // set material buffer
        material.uniform.time = time.elapsed_time;
        // vec4, vec4, f32, f32, f32, u32, f32, XXX
        render_queue.write_buffer(
            &material.buffer,
            0,
            material.uniform.as_std140().as_bytes(),
        );
    }
}
