# Rust packet visualizer
An alternative way to visualize how packets on an interface form a graph. Defaults to listening on interface "en0"

![packetvisualizer](https://user-images.githubusercontent.com/29875928/189706365-64882191-53b9-469e-a7d3-7709c2f60df0.gif)

## Requirements
### Mac
- homebrew
- sdl2
    -  ```brew search sdl2```
    -  ```brew install sdl2 sdl2_image sdl2_mixer sdl2_net sdl2_ttf...```
    - You may need to ensure these are on your path so cargo run can find them
        - https://github.com/PistonDevelopers/rust-empty/issues/175
### Other OS
- idk

## Installation

Once you have the requirements, use ```Cargo run``` to run the project.

## Possible improvements
- network hotspots
- Clicking on a node for more information
- Assets for drawing circles
- Easier way to specify which interface to listen on
