use crate::error::EntityParseError;
use crate::Vector;
use binrw::BinRead;
use std::fmt;
use std::fmt::Debug;
use std::str::FromStr;
use vbsp_derive::Entity;

#[derive(Clone)]
pub struct Entities {
    pub entities: String,
}

impl fmt::Debug for Entities {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        #[derive(Debug)]
        struct Entities<'a> {
            #[allow(dead_code)]
            entities: Vec<RawEntity<'a>>,
        }

        Entities {
            entities: self.iter().collect(),
        }
        .fmt(f)
    }
}

impl Entities {
    pub fn iter(&self) -> impl Iterator<Item = RawEntity<'_>> {
        struct Iter<'a> {
            buf: &'a str,
        }

        impl<'a> Iterator for Iter<'a> {
            type Item = RawEntity<'a>;

            fn next(&mut self) -> Option<Self::Item> {
                let start = self.buf.find('{')? + 1;
                let end = start + self.buf[start..].find('}')?;

                let out = &self.buf[start..end];

                self.buf = &self.buf[end + 1..];

                Some(RawEntity { buf: out })
            }
        }

        Iter {
            buf: &self.entities,
        }
    }
}

#[derive(Clone)]
pub struct RawEntity<'a> {
    buf: &'a str,
}

impl fmt::Debug for RawEntity<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::collections::HashMap;

        self.properties().collect::<HashMap<_, _>>().fmt(f)
    }
}

impl<'a> RawEntity<'a> {
    pub fn properties(&self) -> impl Iterator<Item = (&'a str, &'a str)> {
        struct Iter<'a> {
            buf: &'a str,
        }

        impl<'a> Iterator for Iter<'a> {
            type Item = (&'a str, &'a str);

            fn next(&mut self) -> Option<Self::Item> {
                let start = self.buf.find('"')? + 1;
                let end = start + self.buf[start..].find('"')?;

                let key = &self.buf[start..end];

                let rest = &self.buf[end + 1..];

                let start = rest.find('"')? + 1;
                let end = start + rest[start..].find('"')?;

                let value = &rest[start..end];

                self.buf = &rest[end + 1..];

                Some((key, value))
            }
        }

        Iter { buf: self.buf }
    }

    pub fn prop(&self, key: &'static str) -> Result<&'a str, EntityParseError> {
        self.properties()
            .find_map(|(prop_key, value)| (key == prop_key).then_some(value))
            .ok_or(EntityParseError::NoSuchProperty(key))
    }

    fn prop_parse<T: EntityProp<'a>>(&self, key: &'static str) -> Result<T, EntityParseError> {
        T::parse(self.prop(key)?)
    }

    pub fn parse(&self) -> Result<Entity<'a>, EntityParseError> {
        self.clone().try_into()
    }
}

trait EntityProp<'a>: Sized {
    fn parse(raw: &'a str) -> Result<Self, EntityParseError>;
}

trait FromStrProp: FromStr {}

impl FromStrProp for u8 {}
impl FromStrProp for f32 {}
impl FromStrProp for u32 {}
impl FromStrProp for i32 {}
impl FromStrProp for Vector {}

impl<T: FromStrProp> EntityProp<'_> for T
where
    EntityParseError: From<<T as FromStr>::Err>,
{
    fn parse(raw: &'_ str) -> Result<Self, EntityParseError> {
        Ok(raw.parse()?)
    }
}

impl<T: FromStrProp, const N: usize> EntityProp<'_> for [T; N]
where
    EntityParseError: From<<T as FromStr>::Err>,
    [T; N]: Default,
{
    fn parse(raw: &'_ str) -> Result<Self, EntityParseError> {
        let mut values = raw.split(' ').map(T::from_str);
        let mut result = <[T; N]>::default();
        for item in result.iter_mut() {
            *item = values.next().ok_or(EntityParseError::ElementCount)??;
        }
        Ok(result)
    }
}

impl<'a> EntityProp<'a> for &'a str {
    fn parse(raw: &'a str) -> Result<Self, EntityParseError> {
        Ok(raw)
    }
}

impl EntityProp<'_> for bool {
    fn parse(raw: &'_ str) -> Result<Self, EntityParseError> {
        Ok(raw != "0")
    }
}

impl<'a, T: EntityProp<'a>> EntityProp<'a> for Option<T> {
    fn parse(raw: &'a str) -> Result<Self, EntityParseError> {
        Ok(Some(T::parse(raw)?))
    }
}

#[derive(Debug, Clone, Entity)]
pub enum Entity<'a> {
    #[entity(name = "point_spotlight")]
    SpotLight(SpotLight),
    #[entity(name = "light")]
    Light(Light),
    #[entity(name = "light_spot")]
    LightSpot(LightSpot<'a>),
    #[entity(name = "prop_dynamic")]
    PropDynamic(PropDynamic<'a>),
    #[entity(name = "prop_dynamic_override")]
    PropDynamicOverride(PropDynamicOverride<'a>),
    #[entity(name = "prop_physics_multiplayer")]
    PropPhysics(PropDynamic<'a>),
    #[entity(name = "env_sprite")]
    EnvSprite(EnvSprite<'a>),
    #[entity(name = "info_player_teamspawn")]
    Spawn(Spawn<'a>),
    #[entity(name = "func_regenerate")]
    Regenerate(Regenerate<'a>),
    #[entity(name = "func_respawnroom")]
    RespawnRoom(RespawnRoom<'a>),
    #[entity(name = "func_door")]
    Door(Door<'a>),
    #[entity(name = "worldspawn")]
    WorldSpawn(WorldSpawn<'a>),
    #[entity(name = "info_observer_point")]
    ObserverPoint(ObserverPoint<'a>),
    #[entity(name = "func_brush")]
    Brush(BrushEntity<'a>),
    #[entity(name = "item_ammopack_small")]
    AmmoPackSmall(AmmoPack),
    #[entity(name = "item_ammopack_medium")]
    AmmoPackMedium(AmmoPack),
    #[entity(name = "item_ammopack_full")]
    HealthPackFull(HealthPack),
    #[entity(name = "item_healthkit_small")]
    HealthPackSmall(HealthPack),
    #[entity(name = "item_healthkit_medium")]
    HealthPackMedium(HealthPack),
    #[entity(name = "item_healthkit_full")]
    AmmoPackFull(AmmoPack),
    #[entity(name = "env_lightglow")]
    LightGlow(LightGlow),
    #[entity(name = "trigger_multiple")]
    TriggerMultiple(TriggerMultiple<'a>),
    #[entity(name = "logic_relay")]
    LogicRelay(LogicRelay<'a>),
    #[entity(name = "filter_activator_tfteam")]
    FilterActivatorTeam(FilterActivatorTeam<'a>),
    #[entity(name = "logic_auto")]
    LogicAuto(LogicAuto<'a>),
    #[entity(name = "func_dustmotes")]
    DustMotes(DustMotes<'a>),
    #[entity(name = "sky_camera")]
    SkyCamera(SkyCamera),
    #[entity(name = "path_track")]
    PathTrack(PathTrack<'a>),
    #[entity(name = "env_soundscape_proxy")]
    SoundScapeProxy(SoundScapeProxy<'a>),
    #[entity(name = "func_respawnroomvisualizer")]
    RespawnVisualizer(RespawnVisualizer<'a>),
    #[entity(name = "info_particle_system")]
    ParticleSystem(ParticleSystem<'a>),
    #[entity(name = "team_control_point")]
    TeamControlPoint(TeamControlPoint<'a>),
    #[entity(name = "func_areaportal")]
    AreaPortal(AreaPortal),
    #[entity(name = "game_text")]
    GameText(GameText<'a>),
    #[entity(name = "keyframe_rope")]
    RopeKeyFrame(RopeKeyFrame<'a>),
    #[entity(name = "move_rope")]
    RopeMove(RopeMove<'a>),
    #[entity(name = "tf_gamerules")]
    GameRules(GameRules<'a>),
    #[entity(name = "tf_logic_koth")]
    KothLogic(KothLogic),
    #[entity(default)]
    Unknown(RawEntity<'a>),
}

// TODO: have parsing mode which warns when we skip over fields in an entity!

#[derive(Debug, Clone, Entity)]
pub struct Light {
    pub origin: Vector,
    #[entity(name = "_light")]
    pub light: [u32; 4],
}

#[derive(Debug, Clone, Copy, BinRead)]
#[br(repr(u8))]
pub enum RenderMode {
    Normal = 0,
    Color = 1,
    Texture = 2,
    Glow = 3,
    Solid = 4,
    Additive = 5,
    Unknown = 6,
    AdditiveFractional = 7,
    AlphaAdd = 8,
    WorldSpaceGlow = 9,
    SkipRender = 10,
}
impl EntityProp<'_> for RenderMode {
    fn parse(raw: &str) -> Result<Self, EntityParseError> {
        let val = u8::parse(raw)?;
        Ok(match val {
            0 => RenderMode::Normal,
            1 => RenderMode::Color,
            2 => RenderMode::Texture,
            3 => RenderMode::Glow,
            4 => RenderMode::Solid,
            5 => RenderMode::Additive,
            6 => RenderMode::Unknown,
            7 => RenderMode::AdditiveFractional,
            8 => RenderMode::AlphaAdd,
            9 => RenderMode::WorldSpaceGlow,
            10 => RenderMode::SkipRender,
            _ => return Err(EntityParseError::InvalidEnumValue("RenderMode")),
        })
    }
}

#[derive(Debug, Clone, Entity)]
pub struct SpotLight {
    pub origin: Vector,
    pub angles: [f32; 3],
    #[entity(name = "rendercolor")]
    pub color: [u8; 3],
    /// Width of the spotlight
    #[entity(name = "spotlightwidth")]
    pub cone: u32,
    /// Length of the spotlight
    #[entity(name = "spotlightlength")]
    pub length: u32,
    /// Whether the entity should have shadows on itself
    #[entity(name = "disablereceiveshadows", default)]
    pub disable_receive_shadows: Option<bool>,
    // maybe defaults to 0?
    #[entity(name = "renderfx", default)]
    pub render_fx: Option<u8>,
    // probably defaults to 'Normal'
    #[entity(name = "rendermode", default)]
    pub render_mode: Option<RenderMode>,
}

#[derive(Debug, Default, Clone, Copy, BinRead)]
#[br(repr(u8))]
pub enum LightSpotStyle {
    #[default]
    /// `m` (solid light)
    Normal = 0,
    /// `mmamammmmammamamaaamammma`
    FlourescentFlicker = 10,
    /// `abcdefghijklmnopqrstuvwxyzyxwvutsrqponmlkjihgfedcba`
    SlowStrongPulse = 2,
    /// `abcdefghijklmnopqrrqponmlkjihgfedcba`
    SlowPulseNoBlack = 11,
    /// `jklmnopqrstuvwxyzyxwvutsrqponmlkj`
    GentlePulse = 5,
    /// `mmnmmommommnonmmonqnmmo`
    FlickerA = 1,
    /// `nmonqnmomnmomomno`
    FlickerB = 6,
    /// `mmmmmaaaaammmmmaaaaaabcdefgabcdefg`
    CandleA = 3,
    /// `mmmaaaabcdefgmmmmaaaammmaamm`
    CandleB = 7,
    /// `mmmaaammmaaammmabcdefaaaammmmabcdefmmmaaaa`
    CandleC = 8,
    /// `mamamamamama`
    FastStrobe = 4,
    /// `aaaaaaaazzzzzzzz`
    SlowStrobe = 9,
    /// `mmnnmmnnnmmnn`
    UnderwaterLightMutation = 12,
}
impl EntityProp<'_> for LightSpotStyle {
    fn parse(raw: &str) -> Result<Self, EntityParseError> {
        let val = u8::parse(raw)?;
        Ok(match val {
            0 => LightSpotStyle::Normal,
            10 => LightSpotStyle::FlourescentFlicker,
            2 => LightSpotStyle::SlowStrongPulse,
            11 => LightSpotStyle::SlowPulseNoBlack,
            5 => LightSpotStyle::GentlePulse,
            1 => LightSpotStyle::FlickerA,
            6 => LightSpotStyle::FlickerB,
            3 => LightSpotStyle::CandleA,
            7 => LightSpotStyle::CandleB,
            8 => LightSpotStyle::CandleC,
            4 => LightSpotStyle::FastStrobe,
            9 => LightSpotStyle::SlowStrobe,
            12 => LightSpotStyle::UnderwaterLightMutation,
            _ => return Err(EntityParseError::InvalidEnumValue("LightSpotStyle")),
        })
    }
}

#[derive(Debug, Clone, Entity)]
pub struct LightSpot<'a> {
    pub origin: Vector,
    pub angles: [f32; 3],
    #[entity(name = "_light")]
    pub light: [u32; 4],
    // TODO: hackily set to i32 to allow for -1
    #[entity(name = "_lightHDR")]
    pub light_hdr: [i32; 4],
    #[entity(default)]
    pub style: LightSpotStyle,
    #[entity(default)]
    pub pattern: Option<&'a str>,
    #[entity(name = "_cone")]
    pub cone: u8,
    #[entity(name = "_inner_cone")]
    pub inner_cone: u8,
    #[entity(name = "_exponent")]
    pub exponent: f32,
    /// Max distance light is allowed to cast in inches  
    /// Doesn't work after Source 2013  
    /// (Probably value of zero means ignore it?)
    #[entity(name = "_distance", default)]
    pub distance: u32,
    #[entity(name = "_lightscaleHDR", default)]
    pub light_scale_hdr: Option<f32>,
    /// Used instead of the angles pitch yaw roll
    pub pitch: f32,
    #[entity(name = "_constant_attn", default)]
    pub constant: Option<f32>,
    #[entity(name = "_linear_attn", default)]
    pub linear: Option<f32>,
    #[entity(name = "_quadratic_attn", default)]
    pub quadratic: Option<f32>,
    /// Distance at which brightness should have fallen to 50%  
    /// Overrides linear/constant/quadratic if non-zero
    #[entity(name = "_zero_percent_distance", default)]
    pub zero_percent_distance: f32,
    /// Distance at which brightness should have fallen to (1/256)%  
    /// Overrides linear/constant/quadratic if non-zero
    #[entity(name = "_fifty_percent_distance", default)]
    pub fifty_percent_distance: f32,
}

#[derive(Debug, Clone, Entity)]
pub struct PropDynamic<'a> {
    pub angles: [f32; 3],
    #[entity(name = "disablereceiveshadows", default)]
    pub disable_receive_shadows: bool,
    #[entity(name = "disableshadows", default)]
    pub disable_shadows: bool,
    #[entity(name = "modelscale", default)]
    pub scale: Option<f32>,
    pub model: &'a str,
    pub origin: Vector,
    #[entity(name = "rendercolor")]
    pub color: [u8; 3],
    #[entity(name = "targetname", default)]
    pub name: Option<&'a str>,
    #[entity(name = "parentname", default)]
    pub parent: Option<&'a str>,
}

#[derive(Debug, Clone, Entity)]
pub struct PropDynamicOverride<'a> {
    pub angles: [f32; 3],
    #[entity(name = "disablereceiveshadows", default)]
    pub disable_receive_shadows: bool,
    #[entity(name = "disableshadows", default)]
    pub disable_shadows: bool,
    #[entity(name = "modelscale")]
    pub scale: f32,
    pub model: &'a str,
    pub origin: Vector,
    #[entity(name = "rendercolor")]
    pub color: [u8; 3],
    #[entity(name = "targetname", default)]
    pub name: Option<&'a str>,
    #[entity(name = "parentname", default)]
    pub parent: Option<&'a str>,
}

#[derive(Debug, Clone, Entity)]
pub struct EnvSprite<'a> {
    pub origin: Vector,
    pub scale: f32,
    pub model: &'a str,
    #[entity(name = "rendercolor")]
    pub color: [u8; 3],
}

#[derive(Debug, Clone, Entity)]
pub struct Spawn<'a> {
    pub origin: Vector,
    pub angles: [f32; 3],
    #[entity(name = "targetname", default)]
    pub target: Option<&'a str>,
    #[entity(name = "controlpoint", default)]
    pub control_point: Option<&'a str>,
    #[entity(name = "StartDisabled", default)]
    pub start_disabled: bool,
    #[entity(name = "TeamNum")]
    pub team: u8,
}

#[derive(Debug, Clone, Entity)]
pub struct RespawnRoom<'a> {
    #[entity(name = "targetname", default)]
    pub target: Option<&'a str>,
    pub model: &'a str,
    #[entity(name = "StartDisabled", default)]
    pub start_disabled: bool,
    #[entity(name = "TeamNum")]
    pub team: u8,
}

#[derive(Debug, Clone, Entity)]
pub struct Regenerate<'a> {
    #[entity(name = "associatedmodel")]
    pub associated_model: &'a str,
    pub model: &'a str,
    #[entity(name = "TeamNum")]
    pub team: u8,
}

#[derive(Debug, Clone, Entity)]
pub struct Door<'a> {
    pub origin: Vector,
    #[entity(name = "targetname", default)]
    pub target: &'a str,
    pub speed: f32,
    #[entity(name = "forceclosed", default)]
    pub force_closed: bool,
    #[entity(name = "movedir")]
    pub move_direction: Vector,
    pub model: &'a str,
}

#[derive(Debug, Clone, Entity)]
pub struct AmmoPack {
    pub origin: Vector,
}

#[derive(Debug, Clone, Entity)]
pub struct HealthPack {
    pub origin: Vector,
}

#[derive(Debug, Clone, Entity)]
pub struct WorldSpawn<'a> {
    #[entity(name = "world_mins")]
    pub min: Vector,
    #[entity(name = "world_mins")]
    pub max: Vector,
    #[entity(name = "detailvbsp")]
    pub detail_vbsp: &'a str,
    #[entity(name = "detailmaterial")]
    pub detail_material: &'a str,
    #[entity(default)]
    pub comment: Option<&'a str>,
    #[entity(name = "skyname")]
    pub skybox: &'a str,
    #[entity(name = "mapversion")]
    pub version: u32,
}

#[derive(Debug, Clone, Entity)]
pub struct ObserverPoint<'a> {
    #[entity(name = "StartDisabled", default)]
    pub start_disabled: bool,
    pub angles: [f32; 3],
    pub origin: Vector,
    #[entity(name = "targetname", default)]
    pub target: Option<&'a str>,
    #[entity(name = "parentname", default)]
    pub parent: Option<&'a str>,
}

#[derive(Debug, Clone, Entity)]
pub struct BrushEntity<'a> {
    pub model: &'a str,
    pub origin: Vector,
    #[entity(name = "StartDisabled", default)]
    pub start_disabled: bool,
    #[entity(name = "rendercolor")]
    pub color: [f32; 3],
}

#[derive(Debug, Clone, Entity)]
pub struct LightGlow {
    pub origin: Vector,
    #[entity(name = "VerticalGlowSize")]
    pub vertical_size: u32,
    #[entity(name = "HorizontalGlowSize")]
    pub horizontal_size: u32,
    #[entity(name = "StartDisabled", default)]
    pub start_disabled: bool,
    #[entity(name = "rendercolor")]
    pub color: [f32; 3],
    #[entity(name = "MinDist")]
    pub min_distance: u32,
    #[entity(name = "MaxDist")]
    pub max_distance: u32,
}

#[derive(Debug, Clone, Entity)]
pub struct TriggerMultiple<'a> {
    pub model: &'a str,
    pub origin: Vector,
    #[entity(name = "OnStartTouch", default)]
    pub start_touch: Option<&'a str>,
    #[entity(name = "OnStartTouchAll", default)]
    pub start_touch_all: Option<&'a str>,
    #[entity(name = "OnEndTouch", default)]
    pub end_touch: Option<&'a str>,
    #[entity(name = "OnEndTouchAll", default)]
    pub end_touch_all: Option<&'a str>,
    #[entity(name = "OnNotTouching", default)]
    pub not_touching: Option<&'a str>,
    #[entity(name = "targetname", default)]
    pub target_name: Option<&'a str>,
    #[entity(name = "filtername", default)]
    pub filter: Option<&'a str>,
    pub wait: Option<u32>,
    #[entity(name = "StartDisabled", default)]
    pub start_disabled: bool,
}

#[derive(Debug, Clone, Entity)]
pub struct FilterActivatorTeam<'a> {
    pub origin: Vector,
    #[entity(name = "targetname", default)]
    pub target_name: Option<&'a str>,
    #[entity(name = "negated", default)]
    pub negated: Option<&'a str>,
    #[entity(name = "TeamNum", default)]
    pub team: u8,
}

#[derive(Debug, Clone, Entity)]
pub struct LogicRelay<'a> {
    pub origin: Vector,
    #[entity(name = "targetname", default)]
    pub target_name: Option<&'a str>,
    #[entity(name = "OnTrigger", default)]
    pub on_trigger: Option<&'a str>,
}

#[derive(Debug, Clone, Entity)]
pub struct LogicAuto<'a> {
    pub origin: Vector,
    #[entity(name = "OnMapSpawn", default)]
    pub on_map_spawn: Option<&'a str>,
}

#[derive(Debug, Clone, Entity)]
pub struct DustMotes<'a> {
    pub model: &'a str,
    #[entity(default)]
    pub origin: Option<Vector>,
    #[entity(name = "StartDisabled", default)]
    pub start_disabled: bool,
    #[entity(name = "Color")]
    pub color: [f32; 3],
    #[entity(name = "SpawnRate")]
    pub spawn_rate: u32,
    #[entity(name = "SizeMin")]
    pub size_min: u32,
    #[entity(name = "SizeMax")]
    pub size_max: u32,
    #[entity(name = "Alpha")]
    pub alpha: u8,
}

#[derive(Debug, Clone, Entity)]
pub struct SkyCamera {
    pub origin: Vector,
    #[entity(name = "fogenable")]
    pub fog: bool,
    pub use_angles: bool,
    #[entity(name = "fogstart")]
    pub fog_start: f32,
    #[entity(name = "fogend")]
    pub fog_end: f32,
    pub angles: [u32; 3],
    #[entity(name = "fogdir")]
    pub direction: [u8; 3],
    pub scale: u32,
    #[entity(name = "fogcolor")]
    pub color: [u8; 3],
    #[entity(name = "fogcolor2", default)]
    pub color2: Option<[u8; 3]>,
}

#[derive(Debug, Clone, Entity)]
pub struct PathTrack<'a> {
    pub origin: Vector,
    #[entity(default)]
    pub target: Option<&'a str>,
    #[entity(name = "targetname", default)]
    pub target_name: Option<&'a str>,
    #[entity(name = "orientationtype", default)]
    pub orientation_type: u8,
    pub angles: [u32; 3],
    pub radius: f32,
    pub speed: f32,
}

#[derive(Debug, Clone, Entity)]
pub struct SoundScapeProxy<'a> {
    pub origin: Vector,
    pub radius: f32,
    #[entity(name = "MainSoundscapeName")]
    pub main_name: &'a str,
}

#[derive(Debug, Clone, Entity)]
pub struct RespawnVisualizer<'a> {
    pub origin: Vector,
    #[entity(name = "respawnroomname")]
    pub room_name: &'a str,
    #[entity(name = "rendercolor")]
    pub color: [f32; 3],
    pub solid_to_enemies: bool,
}

#[derive(Debug, Clone, Entity)]
pub struct ParticleSystem<'a> {
    pub origin: Vector,
    pub angles: [f32; 3],
    #[entity(name = "targetname")]
    pub target_name: &'a str,
    pub effect_name: &'a str,
    #[entity(default)]
    pub start_active: bool,
}

#[derive(Debug, Clone, Entity)]
pub struct TeamControlPoint<'a> {
    pub origin: Vector,
    pub angles: [f32; 3],
    #[entity(name = "targetname")]
    pub target_name: &'a str,
    pub point_warn_sound: &'a str,
    pub team_model_0: &'a str,
    pub team_model_2: &'a str,
    pub team_model_3: &'a str,
    pub team_icon_0: &'a str,
    pub team_icon_2: &'a str,
    pub team_icon_3: &'a str,
    pub point_default_owner: u8,
    #[entity(name = "StartDisabled", default)]
    pub start_disabled: bool,
}

#[derive(Debug, Clone, Entity)]
pub struct AreaPortal {
    #[entity(name = "PortalVersion")]
    pub version: u8,
    #[entity(name = "portalnumber")]
    pub number: u8,
    #[entity(name = "StartOpen")]
    pub start_open: bool,
}

#[derive(Debug, Clone, Entity)]
pub struct GameText<'a> {
    pub origin: Vector,
    #[entity(name = "targetname", default)]
    pub target_name: Option<&'a str>,
    pub message: &'a str,
    pub fadeout: f32,
    pub color: [u8; 3],
    #[entity(name = "fadein")]
    pub fade_in: f32,
    #[entity(name = "fadeout")]
    pub fade_out: f32,
    pub x: f32,
    pub y: f32,
    #[entity(name = "holdtime")]
    pub hold_time: f32,
    #[entity(name = "fxtime")]
    pub fx_time: f32,
    pub channel: u8,
}

#[derive(Debug, Clone, Entity)]
pub struct RopeKeyFrame<'a> {
    pub origin: Vector,
    #[entity(name = "targetname", default)]
    pub target_name: Option<&'a str>,
    #[entity(name = "RopeMaterial")]
    pub material: &'a str,
    #[entity(name = "Dangling", default)]
    pub dangling: Option<bool>,
    #[entity(name = "Barbed", default)]
    pub barbed: Option<bool>,
    #[entity(name = "Breakable", default)]
    pub breakable: Option<bool>,
    #[entity(name = "TextureScale")]
    pub texture_scale: f32,
    #[entity(name = "Collide", default)]
    pub collide: Option<bool>,
    #[entity(name = "Width")]
    pub width: f32,
    #[entity(name = "Slack")]
    pub slack: f32,
    #[entity(name = "MoveSpeed")]
    pub move_speed: f32,
    #[entity(name = "Subdiv")]
    pub sub_div: u8,
}

#[derive(Debug, Clone, Entity)]
pub struct RopeMove<'a> {
    pub origin: Vector,
    #[entity(name = "RopeMaterial")]
    pub material: &'a str,
    #[entity(name = "TextureScale")]
    pub texture_scale: f32,
    #[entity(name = "Slack")]
    pub slack: f32,
    #[entity(name = "Width")]
    pub width: f32,
    #[entity(name = "Dangling", default)]
    pub dangling: Option<bool>,
    #[entity(name = "Barbed", default)]
    pub barbed: Option<bool>,
    #[entity(name = "Breakable", default)]
    pub breakable: Option<bool>,
    #[entity(name = "PositionInterpolator")]
    pub interpolator: u8,
    #[entity(name = "MoveSpeed")]
    pub move_speed: f32,
    #[entity(name = "Type", default)]
    pub ty: Option<u8>,
    #[entity(name = "NextKey")]
    pub next_key: &'a str,
    #[entity(name = "Subdiv")]
    pub sub_div: u8,
}

#[derive(Debug, Clone, Entity)]
pub struct GameRules<'a> {
    pub origin: Vector,
    #[entity(name = "targetname", default)]
    pub target_name: Option<&'a str>,
    #[entity(default)]
    pub ctf_overtime: bool,
    #[entity(default)]
    pub hud_type: u32,
}

#[derive(Debug, Clone, Entity)]
pub struct KothLogic {
    pub origin: Vector,
    pub unlock_point: u32,
    pub timer_length: u32,
}
