use base64::{Engine as _, engine::general_purpose::STANDARD};
use log::{debug, error};
use matrix_sdk::{
    Client, Room, RoomMemberships,
    media::{MediaFormat, MediaRequestParameters, MediaThumbnailSettings},
    room::RoomMember,
    ruma::{OwnedMxcUri, events::room::MediaSource, media::Method},
};

/// Avatar size preset mapping display sizes to thumbnail pixel dimensions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum AvatarSize {
    #[default]
    Small, // 32px layout -> 64px thumbnail for 2x HiDPI crispness
    Medium, // 48px layout -> 96px thumbnail for 2x HiDPI crispness
    Large,  // 64px layout -> 128px thumbnail for 2x HiDPI crispness
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

/// Generates standard thumbnail settings for avatars.
pub fn avatar_thumbnail_format(size: impl Into<AvatarSize>) -> MediaFormat {
    let px = size.into().pixels();
    MediaFormat::Thumbnail(MediaThumbnailSettings {
        method: Method::Crop,
        width: px.into(),
        height: px.into(),
        animated: false,
    })
}

/// Encodes raw image bytes into a data URI (`data:<mime>;base64,<data>`).
pub fn encode_to_data_uri(bytes: Vec<u8>) -> Option<String> {
    if bytes.len() > 2_097_152 {
        error!("Avatar image too large: {} bytes", bytes.len());
        return None;
    }
    let mime_type = match infer::get(&bytes) {
        Some(inferred) => inferred.mime_type(),
        None => {
            error!("Could not determine MIME type for avatar bytes");
            return None;
        }
    };
    let b64_string = STANDARD.encode(&bytes);
    Some(format!("data:{};base64,{}", mime_type, b64_string))
}

/// Fetches media thumbnail by MXC URI.
pub async fn get_mxc_avatar(
    client: &Client,
    mxc: &OwnedMxcUri,
    size: impl Into<AvatarSize>,
) -> Option<String> {
    let request = MediaRequestParameters {
        source: MediaSource::Plain(mxc.clone()),
        format: avatar_thumbnail_format(size),
    };
    if let Ok(bytes) = client.media().get_media_content(&request, true).await {
        return encode_to_data_uri(bytes);
    }
    None
}

/// Fetches a room's avatar, falling back to single participant avatar for 1-on-1 DMs if set.
pub async fn get_room_avatar(
    client: &Client,
    room: &Room,
    size: impl Into<AvatarSize>,
) -> Option<String> {
    let target_size = size.into();
    let format = avatar_thumbnail_format(target_size);

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
    let format = avatar_thumbnail_format(size);
    if let Ok(Some(bytes)) = member.avatar(format).await {
        return encode_to_data_uri(bytes);
    }
    None
}

/// Fetches current logged-in user profile avatar thumbnail.
pub async fn get_user_profile_avatar(
    client: &Client,
    size: impl Into<AvatarSize>,
) -> Option<String> {
    let mxc = client.account().get_avatar_url().await.ok().flatten()?;
    get_mxc_avatar(client, &mxc, size).await
}

/// Fetches avatar thumbnail for any user by their `UserId`.
#[allow(deprecated)]
pub async fn get_user_avatar(
    client: &Client,
    user_id: &matrix_sdk::ruma::UserId,
    size: impl Into<AvatarSize>,
) -> Option<String> {
    use matrix_sdk::ruma::api::client::profile::get_avatar_url;
    let request = get_avatar_url::v3::Request::new(user_id.to_owned());
    if let Ok(response) = client.send(request).await
        && let Some(mxc) = response.avatar_url
    {
        return get_mxc_avatar(client, &mxc, size).await;
    }
    None
}
