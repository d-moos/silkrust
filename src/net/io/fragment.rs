use bytes::Bytes;

pub trait Fragment {
    fn get(reader: &mut Bytes) -> Self;
}
