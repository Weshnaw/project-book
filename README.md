# Project Book

This is a WIP cross platform plex audiobook player

## Completed Tasks:
- enable htmx with tauri calls
- basic main tabs
- state management
- scalfold displaying books
- scalfold plex signin
- plex sign in
- hot reloading settings on change
- events for settings refresh
- select plex server

## Upcomming Tasks:
- load library
- download books
- stream audio

## Future Tasks:
- handle chapters
- handle book metadata
- playback speed
- bookmarks
- sleep timer
- better UX
- github actions
- better more thorough testing
- user ability to reset state
- get device info
- better debugging
- better logging
- _"cloud"_ syncing
  - self hosted syncing?
- recently added
- recently listened
- downloaded list
- update plex with read status / progress
- in app logging view / logging files
- async reqwest
- Maybe a scalfolding to allow for mock functionality (only on `#[cfg(debug_assertions)]` see [example](https://stackoverflow.com/questions/39204908/how-to-check-release-debug-builds-using-cfg-in-rust))
- macro/trait store functions
- is it fine to ignore save errors? (ie `store.save().ok();`)
- Shared client for plex
  - Custom deserializer?

## Recommended IDE Setup for Tauri

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
