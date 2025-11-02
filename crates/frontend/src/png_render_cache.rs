use std::{sync::Arc, time::Duration};

use gpui::RenderImage;
use image::Frame;
use once_cell::sync::Lazy;

static STATE: Lazy<PngRenderCache> = Lazy::new(PngRenderCache::new);

struct PngRenderCache {
    by_ptr: mini_moka::sync::Cache<usize, Option<Arc<RenderImage>>>,
    by_arc: mini_moka::sync::Cache<Arc<[u8]>, Option<Arc<RenderImage>>>,
}

impl PngRenderCache {
    pub fn new() -> Self {
        Self {
            by_ptr: mini_moka::sync::Cache::builder().time_to_idle(Duration::from_secs(60)).build(),
            by_arc: mini_moka::sync::Cache::builder().time_to_idle(Duration::from_secs(60)).build(),
        }
    }
}


pub fn render(image: Arc<[u8]>) -> gpui::Img {
    if let Some(result) = get_render_image(image) {
        return gpui::img(result);
    } else {
        return gpui::img(gpui::ImageSource::Resource(gpui::Resource::Embedded("images/missing.png".into())));
    }
}


pub fn get_render_image(image: Arc<[u8]>) -> Option<Arc<RenderImage>> {
    let cache = &*STATE;
    
    let ptr = Arc::as_ptr(&image).addr();
    
    if let Some(result) = cache.by_ptr.get(&ptr) {
        return result;
    }
    
    if let Some(result) = cache.by_arc.get(&image) {
        cache.by_ptr.insert(ptr, result.clone());
        return result;
    }
    
    let result = image::load_from_memory_with_format(&*image, image::ImageFormat::Png).map(|data| {
        let mut data = data.into_rgba8();
        
        // Convert from RGBA to BGRA.
        for pixel in data.chunks_exact_mut(4) {
            pixel.swap(0, 2);
        }
        
        RenderImage::new([Frame::new(data)])
    });
    
    let render_image = match result {
        Ok(render_image) => {
            Some(Arc::new(render_image))
        },
        Err(error) => {
            eprintln!("Error loading png: {error:?}");
            None
        },
    };
    
    cache.by_ptr.insert(ptr, render_image.clone());
    cache.by_arc.insert(Arc::clone(&image), render_image.clone());
    
    render_image
}
