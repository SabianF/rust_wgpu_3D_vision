# rust_wgpu_3D_vision
A program to simulate stereoscopic 4D vision, using a virtual 3D retina which is created by rapidly displaying all voxels at varying depths of a 3D volume.

<details>
<summary>Main Hypothesis</summary>

If we display a 3D object on a 2D screen by rendering multiple layers (or voxels) of its outer and internal textures (like an MRI) fast enough to refresh all voxels composing the object 10-20 times per second (aka VPS or volumes-per-second), then our brains may interpret and process this 3D voxel space as true 3D vision.

**Reasoning:** The voxels would simulate 4D photons hitting a 3D retina, and the brain should process this the same as it's already doing for the 3D photons hitting your 2D retinas as you're reading this.
  
Essentially, this is using time (via [flicker fusion](https://en.wikipedia.org/wiki/Flicker_fusion_threshold)) to extend our 2-dimensional vision into 3D.

If this works, then not only will the brain perceive true 3D volume vision, but also we can create two virtual 3D screens which display from two different 4D angles (simulating two 3D retinas, or two 4D eyes), which the brain may process into 4D binocular vision, letting us perceive 4D parallax & depth.
</details>

Since this is my very first Rust project, the major steps to accomplishing this are

### Phase 1: Core functionality
- [x] Creating a functional window
- [x] Rendering a colour
- [x] Rendering a 2D surface (~~square~~ triangle)
- [x] Rendering a voxel (cube)
- [ ] Rendering a 3D volume containing multiple voxels
- [ ] Flickering the voxels at 1 layer of voxels per frame

### Phase 2: 3D angle-viewing
- [ ] Adding the ability to rotate the 3D volume in 3-space
- [ ] Ensuring all displayed voxels are fully visible at all times (voxels never obscure any part of any other voxels)

### Phase 3: 4D angle-viewing
- [ ] TBD

### Phase 4: 4D objects
- [ ] TBD

# Updates

## From previous repo

1. Started following the incredibly difficult-to-understand Dr. Xu's guide up to video 3
1. Encountered errors ([eg1](https://stackoverflow.com/questions/18004993/how-to-determine-cause-of-directx-11-driver-hang), [eg2](https://www.gamedev.net/forums/topic/703795-dxr-and-device-hung-error/))
1. I have no clue what the error (or help thread comments) mean, so I'm trying [a different tutorial](https://github.com/peerhenry/rust_hello_triangle)
1. I've reverted back to launching a functional empty window, and am now following the [official WGPU tutorial](https://sotrh.github.io/learn-wgpu/beginner/tutorial1-window/#the-code)

## From this repo
1. **done**: Created resizeable window ([commit](https://github.com/SabianF/rust_wgpu_3D_vision/commit/094a5c9e4df79707d4df8df3e0bc1d2aa69d64f7))
1. **done**: rendering colour ([commit](https://github.com/SabianF/rust_wgpu_3D_vision/commit/93f3ad42ea52b5713723b7eed49beac66c95aa25))
   - <img src="https://user-images.githubusercontent.com/58588133/221382461-0ab01c86-9603-4a15-aa18-92feb14675d9.png" width="256" />
1. **done**: rendering 2D surface ([commit](https://github.com/SabianF/rust_wgpu_3D_vision/commit/ad066599d1c539dd1ce8ff6e829685ac643bc246))
   - <img src="https://user-images.githubusercontent.com/58588133/221392828-99132655-2af0-4dca-bf61-5c1958d327b7.png" width="256" />
   - Bonus: added temporary colour switching functionality (didn't commit)
      - <img src="https://user-images.githubusercontent.com/58588133/221487748-ec90ceaa-b4f1-4fe6-8f0f-eeb0a1112a4b.png" width="256" />
1. **done**: rendering 3D object
   - <img src="https://user-images.githubusercontent.com/58588133/221772063-2e042702-97d3-44c8-8037-69c75213bb1e.gif" width="256" alt="rust_cube_rotating" />
1. todo: optimizing camera controls
