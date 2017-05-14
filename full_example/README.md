softrender example project
==========================

![Suzanne Example](example.png)

And with the normals visualized using the geometry shader:

![Suzanne Example Normal Visualizations](example2.png)

A relatively simple project demonstrating softrender's usage of meshes, uniforms, shaders and so forth.

The shading is done with simple Lambertian diffuse and Blinn-Phong specular reflections of points lights.

To run: `cargo run --bin main --release`