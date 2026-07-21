use base64::{Engine as _, engine::general_purpose::STANDARD};
use log::{debug, error};
use matrix_sdk::{
    Client, Room, RoomMemberships,
    media::{MediaFormat, MediaRequestParameters, MediaThumbnailSettings},
    room::RoomMember,
    ruma::{
        OwnedMxcUri, api::client::profile::AvatarUrl, events::room::MediaSource, media::Method,
    },
};

/// Resize method for thumbnail generation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ResizeMethod {
    #[default]
    Scale, // Preserves aspect ratio (ideal for photos & attachments)
    Crop, // Crops to fill exact dimensions (ideal for avatars)
}

/// Avatar size preset mapping display sizes to thumbnail pixel dimensions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum AvatarSize {
    #[default]
    Small, // 64px thumbnail
    Medium, // 96px thumbnail
    Large,  // 128px thumbnail
    Custom(u32),
}

impl AvatarSize {
    pub fn pixels(self) -> u32 {
        match self {
            AvatarSize::Small => 64,
            AvatarSize::Medium => 96,
            AvatarSize::Large => 128,
            AvatarSize::Custom(px) => px,
        }
    }
}

impl From<u32> for AvatarSize {
    fn from(px: u32) -> Self {
        AvatarSize::Custom(px)
    }
}

impl From<AvatarSize> for MediaFormat {
    fn from(size: AvatarSize) -> Self {
        let px = size.pixels();
        MediaFormat::Thumbnail(MediaThumbnailSettings {
            method: Method::Crop,
            width: px.into(),
            height: px.into(),
            animated: false,
        })
    }
}

/// Generic image thumbnail size presets for non-avatar images.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageSize {
    VerySmall, //100 px
    Small,     // 240 px
    Medium,    // 480 px
    Large,     // 768 px
    Custom {
        width: u32,
        height: u32,
        method: ResizeMethod,
    },
}

impl From<ImageSize> for MediaFormat {
    fn from(size: ImageSize) -> Self {
        match size {
            ImageSize::VerySmall => MediaFormat::Thumbnail(MediaThumbnailSettings {
                method: Method::Scale,
                width: 80u32.into(),
                height: 80u32.into(),
                animated: false,
            }),
            ImageSize::Small => MediaFormat::Thumbnail(MediaThumbnailSettings {
                method: Method::Scale,
                width: 240u32.into(),
                height: 240u32.into(),
                animated: false,
            }),
            ImageSize::Medium => MediaFormat::Thumbnail(MediaThumbnailSettings {
                method: Method::Scale,
                width: 480u32.into(),
                height: 480u32.into(),
                animated: false,
            }),
            ImageSize::Large => MediaFormat::Thumbnail(MediaThumbnailSettings {
                method: Method::Scale,
                width: 768u32.into(),
                height: 768u32.into(),
                animated: false,
            }),
            ImageSize::Custom {
                width,
                height,
                method,
            } => MediaFormat::Thumbnail(MediaThumbnailSettings {
                method: match method {
                    ResizeMethod::Scale => Method::Scale,
                    ResizeMethod::Crop => Method::Crop,
                },
                width: width.into(),
                height: height.into(),
                animated: false,
            }),
        }
    }
}

/// Encodes raw image bytes into a data URI (`data:<mime>;base64,<data>`).
fn encode_to_data_uri(bytes: Vec<u8>) -> Option<String> {
    if bytes.len() > 2_097_152 {
        error!("Media image too large: {} bytes", bytes.len());
        return None;
    }
    let mime_type = match infer::get(&bytes) {
        Some(inferred) => inferred.mime_type(),
        None => {
            error!("Could not determine MIME type for media bytes");
            return None;
        }
    };
    let b64_string = STANDARD.encode(&bytes);
    Some(format!("data:{};base64,{}", mime_type, b64_string))
}

/// Trait for converting types (like `OwnedMxcUri` or `MediaSource`) into a `MediaSource`.
pub trait IntoMediaSource {
    fn into_media_source(self) -> MediaSource;
}

impl IntoMediaSource for MediaSource {
    fn into_media_source(self) -> MediaSource {
        self
    }
}

impl IntoMediaSource for &MediaSource {
    fn into_media_source(self) -> MediaSource {
        self.clone()
    }
}

impl IntoMediaSource for OwnedMxcUri {
    fn into_media_source(self) -> MediaSource {
        MediaSource::Plain(self)
    }
}

impl IntoMediaSource for &OwnedMxcUri {
    fn into_media_source(self) -> MediaSource {
        MediaSource::Plain(self.clone())
    }
}

impl IntoMediaSource for matrix_sdk::ruma::events::sticker::StickerMediaSource {
    fn into_media_source(self) -> MediaSource {
        MediaSource::from(self)
    }
}

impl IntoMediaSource for &matrix_sdk::ruma::events::sticker::StickerMediaSource {
    fn into_media_source(self) -> MediaSource {
        MediaSource::from(self.clone())
    }
}

/// Fetches raw media bytes from any `MediaSource` (plain MXC URI or encrypted file).
pub async fn fetch_media_bytes(
    client: &Client,
    source: impl IntoMediaSource,
    format: impl Into<MediaFormat>,
) -> Option<Vec<u8>> {
    let request = MediaRequestParameters {
        source: source.into_media_source(),
        format: format.into(),
    };
    client.media().get_media_content(&request, true).await.ok()
}

/// Fetches media content from any `MediaSource` and encodes it into a base64 data URI.
pub async fn fetch_media_data_uri(
    client: &Client,
    source: impl IntoMediaSource,
    format: impl Into<MediaFormat>,
) -> Option<String> {
    let bytes = fetch_media_bytes(client, source, format).await?;
    encode_to_data_uri(bytes)
}

/// Fetches full-resolution media content as a data URI (`data:<mime>;base64,<data>`).
pub async fn get_media_data_uri(client: &Client, source: impl IntoMediaSource) -> Option<String> {
    fetch_media_data_uri(client, source, MediaFormat::File).await
}

/// Fetches media thumbnail content as a data URI.
pub async fn get_media_thumbnail_data_uri(
    client: &Client,
    source: impl IntoMediaSource,
    format: impl Into<MediaFormat>,
) -> Option<String> {
    fetch_media_data_uri(client, source, format).await
}

/// Fetches media thumbnail by MXC URI.
pub async fn get_mxc_avatar(
    client: &Client,
    mxc: &OwnedMxcUri,
    size: impl Into<AvatarSize>,
) -> Option<String> {
    get_media_thumbnail_data_uri(client, mxc, size.into()).await
}

/// Fetches a room's avatar, falling back to single participant avatar for 1-on-1 DMs if set.
pub async fn get_room_avatar(
    client: &Client,
    room: &Room,
    size: impl Into<AvatarSize>,
) -> Option<String> {
    let format: MediaFormat = size.into().into();

    if let Ok(Some(bytes)) = room.avatar(format.clone()).await {
        return encode_to_data_uri(bytes);
    }

    if let Ok(members) = room.members_no_sync(RoomMemberships::JOIN).await
        && let Some(my_user_id) = client.user_id()
    {
        let other_members: Vec<_> = members
            .into_iter()
            .filter(|m| m.user_id() != my_user_id)
            .collect();

        if other_members.len() == 1 {
            let other_user = &other_members[0];
            debug!("Using fallback avatar from user: {}", other_user.user_id());
            if let Ok(Some(bytes)) = other_user.avatar(format).await {
                return encode_to_data_uri(bytes);
            }
        }
    }

    None
}

/// Fetches a room member's avatar thumbnail.
pub async fn get_member_avatar(member: &RoomMember, size: impl Into<AvatarSize>) -> Option<String> {
    let format: MediaFormat = size.into().into();
    if let Ok(Some(bytes)) = member.avatar(format).await {
        return encode_to_data_uri(bytes);
    }
    None
}

/// Fetches avatar thumbnail for any user by their `UserId`.
pub async fn get_user_avatar(
    client: &Client,
    user_id: &matrix_sdk::ruma::UserId,
    size: impl Into<AvatarSize>,
) -> Option<String> {
    let avatar_url = client
        .account()
        .fetch_profile_field_of_static::<AvatarUrl>(user_id.to_owned())
        .await;
    if avatar_url.is_err() {
        return None;
    }
    let avatar_url = avatar_url.unwrap();
    return get_media_thumbnail_data_uri(client, avatar_url?, size.into()).await;
}
