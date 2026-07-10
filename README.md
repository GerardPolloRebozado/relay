# Relay
Relay is a multiplatform [matrix client](https://matrix.org/), it's still in early stages of development, you can check the current implemented features in the following list:

[x] :lock: E2E encryption
[x] :left_speech_bubble: read and send messages including images in dms, groups and spaces 
[] :video_camera: VoIP calls
[x] :heavy_plus_sign: Room creation
[x] :heavy_minus_sign: Room leaving
[] :crown: Modification of room settings
[x] :information_source: Basic room information (participants, name and room picture)
[] :framed_picture: sticker support
[] :bell: background notifications

You can install the app from the pre-compiled versions on the releases tab.
Take into account relay is in very early stage of developemnt so many features are not implemented or not working properly.

### Serving the Relay during development

Run the following command in the root of your project to start developing with the default platform:

```bash
dx serve
```

To run for a different platform, use the `--platform platform` flag. E.g.
```bash
dx serve --platform desktop
```

# Useful links during development

https://docs.rs/matrix-sdk/latest/matrix_sdk

https://github.com/matrix-org/matrix-rust-sdk/tree/main/examples

https://dioxuslabs.com/learn/0.7/getting_started/

#Components
For components on this project we use oficial dioxus component library and style it to follow our design
https://dioxuslabs.github.io/dioxus-components/
