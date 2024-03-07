pub mod impls;

pub trait ZKChannel<T> {
    fn extend_vole(&mut self, n: u64);
    fn send_delta(&mut self, delta: T);
}
