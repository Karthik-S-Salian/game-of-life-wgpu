// use winit::{application::ApplicationHandler, dpi::Size, event::WindowEvent, event_loop::ActiveEventLoop, window::{Window, WindowAttributes, WindowId}};
// use anyhow::{Ok, Result};
// use crate::{render::Renderer, Config};

// pub struct WindowProperties{
//     window:Window,
//     renderer:Renderer,
//     surface:wgpu::Surface
// }

// pub struct App {
//     window: Option<Window>,
//     renderer:Option<Renderer>,
//     config:Config
// }

// impl App {
//     pub fn new() -> App {
//         let config = Config {
//             width: 800,
//             height: 800,
//             grid_size: 32,
//         };

//         App{
//             config,
//             window:None,
//             renderer:None
//         }
//     }

//     async fn init(&mut self, event_loop: &ActiveEventLoop)-> Result<()>{
//         let window_size = winit::dpi::PhysicalSize::new(self.config.width, self.config.height);


//         self.window = Some(
//             event_loop
//                 .create_window(WindowAttributes{
//                     inner_size:Some(Size::new(window_size)), 
//                     visible:true,
//                     ..Default::default()})
//                 .unwrap(),
//         );

//         if let Some(window)=&self.window{
//             let (device, queue, surface) = connect_to_gpu(window).await?;
//             self.renderer = Some(Renderer::new(device, queue, &self.config));
//         }

//         Ok(())
//     }
// }

// impl ApplicationHandler for App {
//     fn resumed(&mut self, event_loop: &ActiveEventLoop) {

        
//     }

//     fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
//         match event {
//             WindowEvent::CloseRequested => {
//                 println!("The close button was pressed; stopping");
//                 event_loop.exit();
//             }
//             WindowEvent::RedrawRequested => {
//                 let frame = surface
//                     .get_current_texture()
//                     .expect("failed to get current texture");
//                 let render_target = frame
//                     .texture
//                     .create_view(&wgpu::TextureViewDescriptor::default());
//                 self.renderer.render_frame(&render_target);
//                 frame.present();
//                 self.window.as_ref().unwrap().request_redraw();
//             }
//             _ => (),
//         }
//     }
// }


// async fn connect_to_gpu(window: &Window) -> Result<(wgpu::Device, wgpu::Queue, wgpu::Surface)> {
//     use wgpu::TextureFormat::{Bgra8Unorm, Rgba8Unorm};

//     // Create an "instance" of wgpu. This is the entry-point to the API
//     let instance = wgpu::Instance::default();

//     // Create a drawable "surface" that is associated with the window.
//     let surface = instance.create_surface(window)?;

//     // Request a GPU that is compatible with the surface. If the system has multiple GPUs then
//     // pick the high performance one.
//     let adapter = instance
//         .request_adapter(&wgpu::RequestAdapterOptions {
//             power_preference: wgpu::PowerPreference::default(),
//             force_fallback_adapter: false,
//             compatible_surface: Some(&surface),
//         })
//         .await
//         .context("failed to find compatible adapter")?;

//     // Connect to the GPU. "device" represents the connection to the GPU and allows us to create
//     // resources like buffers, textures, and pipelines. "queue" represents the command queue that
//     // we use to submit commands to the GPU.
//     let (device, queue) = adapter
//         .request_device(&wgpu::DeviceDescriptor::default(), None)
//         .await
//         .context("failed to connect to the GPU")?;

//     // Configure the texture memory backs the surface. Our renderer will draw to a surface texture
//     // every frame.
//     let caps = surface.get_capabilities(&adapter);
//     let format = caps
//         .formats
//         .into_iter()
//         .find(|it| matches!(it, Rgba8Unorm | Bgra8Unorm))
//         .context("could not find prefered texture format (Rgba8Unorm or")?;

//     let size = window.inner_size();

//     let config = wgpu::SurfaceConfiguration {
//         usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
//         format,
//         width: size.width,
//         height: size.height,
//         present_mode: wgpu::PresentMode::AutoVsync,
//         alpha_mode: caps.alpha_modes[0],
//         view_formats: vec![],
//         desired_maximum_frame_latency: 3,
//     };

//     surface.configure(&device, &config);

//     Ok((device, queue, surface))
// }