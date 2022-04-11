pub trait ImageGenerator: Clone + Send {
    type ImageDescriptor: Clone + Send + PartialEq + 'static;

    fn new(width: usize, height: usize) -> Self;

    fn do_compute(&mut self, settings: Self::ImageDescriptor, threads: usize);

    fn do_composite(&mut self, settings: Self::ImageDescriptor, threads: usize);

    fn get_progress(&self) -> f64;

    fn width(&self) -> usize;
    fn height(&self) -> usize;

    fn image_data(&self) -> &[u8];

    fn needs_recompute(
        settings: &Self::ImageDescriptor,
        old_settings: &Self::ImageDescriptor,
    ) -> bool;
}
