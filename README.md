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
- _cloud_ syncing
  - self hosted syncing?
- recently added
- recently listened
- downloaded list
- update plex with read status / progress
- in app logging view / logging files
- async reqwest
- Maybe a scalfolding to allow for mock functionality (only on `#[cfg(debug_assertions)]` see [example](https://stackoverflow.com/questions/39204908/how-to-check-release-debug-builds-using-cfg-in-rust))
- better refreshing when data changes
  - we should emit events instead of polling [tauri events?](https://v2.tauri.app/develop/calling-frontend/#event-system)

## Recommended IDE Setup for Tauri

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
