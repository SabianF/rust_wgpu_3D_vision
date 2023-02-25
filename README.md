# rust_wgpu_3D_vision
A program to simulate stereoscopic 4D vision, using a virtual 3D retina which is created by rapidly displaying all voxels at varying depths of a 3D volume.

<details>
<summary>Main Hypothesis</summary>

If we display a 3D object on a 2D screen by rendering multiple layers (or voxels) of its outer and internal textures (like an MRI) fast enough to refresh all voxels composing the object 10-20 times per second (aka VPS or volumes-per-second), then our brains may interpret and process this 3D voxel space as true 3D vision.

**Reasoning:** This would simulate photons hitting a 3D retina, and the brain should adapt and interpret this the same as it's already doing for the photons hitting your 2D retinas as you're reading this.

If this works, then we can create 2 stereoscopic volumes shown from two different 4D angles (simulating two 3D retinas, or two 4D eyes), which the brain may interpret as 4D binocular vision, letting us perceive 4D depth.
</details>

Since this is my very first Rust project, the major steps to accomplishing this are

### Phase 1: Core functionality
- [x] Creating a functional window
- [ ] Rendering a colour
- [ ] Rendering a 2D surface (square)
- [ ] Rendering a voxel (cube)
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
1. Created resizeable window ([commit](https://github.com/SabianF/rust_wgpu_3D_vision/commit/094a5c9e4df79707d4df8df3e0bc1d2aa69d64f7))
1. todo: rendering colour
